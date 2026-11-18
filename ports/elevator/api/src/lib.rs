//! Elevator para o Horizon: pular sobe para o elevador de cima, agachar desce para o de baixo.
//!
//! Port do ElevatorMod (VsnGamer, MIT; veja NOTICE.md). A lógica pura está em `../../src/lib.rs`
//! (testada sem o jogo); este arquivo só liga essa lógica aos ganchos do Horizon.
//!
//! O desenho segue o mod original, que tem duas metades:
//!
//! * **Cliente** (`ElevatorHandler.java`): a cada tick vê se o jogador acabou de apertar pular ou
//!   agachar, procura o elevador de destino e manda um pedido ao servidor.
//! * **Servidor** (`TeleportPacket.java`): valida o pedido, cobra o XP (se configurado) e teleporta.
//!
//! Como no original, o mod precisa estar instalado no cliente e no servidor.
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use horizon_client::mod_api::{
    Flow, Hooks, Host, LevelRef, MovementInputEvent, NativeServerPlayer, NetFromClientEvent,
    PotentialSpawnsEvent, Tag, Text, TickEvent,
};

// Parte da lógica (elevador direcional) ainda não é usada pelo jogo.
#[allow(dead_code)]
#[path = "../../src/lib.rs"]
mod logic;

use logic::{BlockPos, Config, Dir, DyeColor, Elevator, Facing, Pitch, Player, Reject, World, Yaw};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod::default())
}

/// Som tocado no destino: o mesmo `elevatorid:teleport` do ElevatorMod, que vai dentro do `.hmod`
/// (`hmod/assets/elevatorid/sounds.json` e `sounds/teleport.ogg`).
const TELEPORT_SOUND: &str = "elevatorid:teleport";

/// Vermelho (`ChatFormatting.RED`).
const RED: u32 = 0xFF5555;

#[derive(Default)]
struct Mod {
    /// Teclas como estavam no último `movement_input` (cliente).
    jump: AtomicBool,
    sneak: AtomicBool,
    /// Teclas como estavam no tick anterior, para achar o instante em que foram apertadas.
    last_jump: AtomicBool,
    last_sneak: AtomicBool,
    config: OnceLock<Config>,
}

/// Altura do mundo por dimensão (mínimo, altura), quando o jogo não informa (cliente).
fn dimension_height(dimension: &str) -> (i32, i32) {
    match dimension {
        "minecraft:the_nether" | "minecraft:the_end" => (0, 256),
        _ => (-64, 384),
    }
}

/// O mundo visto pelo `Host`: o nível completo no servidor, o mundo em cena no cliente.
struct Reader<'a> {
    host: &'a dyn Host,
    level: Option<LevelRef>,
    /// (y mínimo, altura)
    height: (i32, i32),
}

fn arr(p: BlockPos) -> [i32; 3] {
    [p.x, p.y, p.z]
}

impl World for Reader<'_> {
    fn elevator_at(&self, pos: BlockPos) -> Option<Elevator> {
        let name = match self.level {
            Some(level) => self.host.level_block(level, arr(pos))?,
            None => self.host.block(arr(pos)),
        };
        let color = DyeColor::from_block_name(&name)?;
        // Altura da forma do bloco, para o jogador ficar em cima dele.
        let top = self.host.block_collision(arr(pos)).iter().map(|b| b[4]).fold(0.0_f64, f64::max);
        Some(Elevator {
            color,
            // O estado direcional e a camuflagem dependem de uma entidade de bloco, ainda não portada.
            directional: false,
            facing: Facing::South,
            support_height: if top > 0.0 { top } else { 1.0 },
        })
    }

    fn is_suffocating(&self, pos: BlockPos) -> bool {
        // Sufoca quem está dentro: um bloco com colisão de cubo inteiro.
        self.host.block_collision(arr(pos)).iter().any(|b| {
            b[3] - b[0] >= 0.999 && b[4] - b[1] >= 0.999 && b[5] - b[2] >= 0.999
        })
    }

    fn in_build_height(&self, y: i32) -> bool {
        y >= self.height.0 && y < self.height.0 + self.height.1
    }

    fn is_loaded(&self, pos: BlockPos) -> bool {
        self.level.is_none_or(|level| self.host.level_is_loaded(level, arr(pos)))
    }
}

/// Nomes de jogador válidos: letras, números e `_`. Barra qualquer coisa que possa virar outro comando.
fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 16 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn floor_pos(p: [f64; 3]) -> BlockPos {
    BlockPos::new(p[0].floor() as i32, p[1].floor() as i32, p[2].floor() as i32)
}

impl Mod {
    /// A configuração, lida uma vez de `config/elevatorid-synced.toml` (como o original).
    fn config(&self, host: &dyn Host) -> &Config {
        self.config.get_or_init(|| {
            let path = host.instance_dir().join("config").join("elevatorid-synced.toml");
            std::fs::read_to_string(path).map(|t| Config::from_toml(&t)).unwrap_or_default()
        })
    }

    // ---------------------------------------------------------------- cliente

    /// `ElevatorHandler.tryTeleport`: procura o destino e manda o pedido ao servidor.
    fn client_try(&self, host: &mut dyn Host, dir: Dir) {
        let Some(player) = host.player() else {
            return;
        };
        let cfg = self.config(&*host).clone();
        let here = floor_pos([player.x, player.y, player.z]);
        let request = {
            let world = Reader { host: &*host, level: None, height: dimension_height(player.dimension) };
            logic::origin_elevator(&world, here, &cfg).and_then(|from| {
                logic::find_target(&world, from, dir, &cfg).map(|to| (from, to))
            })
        };
        if let Some((from, to)) = request {
            let _ = host.net_send_to_server(logic::CHANNEL, &logic::encode_request(from, to));
        }
    }

    // --------------------------------------------------------------- servidor

    /// O XP do jogador como o servidor o guarda: nível e progresso (`XpLevel`, `XpP`).
    fn read_xp(host: &mut dyn Host, p: &NativeServerPlayer) -> Option<(i32, f32)> {
        let id = u32::try_from(p.entity_id).ok().filter(|&id| id > 0)?;
        let entity = host.entity(id)?;
        let nbt = host.entity_nbt(entity)?;
        let level = match nbt.get(&"XpLevel".into())? {
            Tag::Int(v) => *v,
            _ => return None,
        };
        let progress = match nbt.get(&"XpP".into())? {
            Tag::Float(bits) => f32::from_bits(*bits),
            _ => return None,
        };
        Some((level, progress))
    }

    /// `TeleportPacket.handle`: valida, cobra XP e teleporta.
    fn server_teleport(&self, host: &mut dyn Host, uuid: u128, from: BlockPos, to: BlockPos) {
        let cfg = self.config(&*host).clone();
        let Some(p) = host.server_players().into_iter().find(|p| p.uuid == uuid) else {
            return;
        };
        if !safe_name(&p.name) {
            return;
        }
        // Os dados de colisão do `Host` valem para o mundo em que o gancho roda.
        if host.dimension() != p.dimension {
            return;
        }
        let Some(level) = host.level(&p.dimension) else {
            return;
        };
        let height = host.level_height(level).unwrap_or_else(|| dimension_height(&p.dimension));

        let here = floor_pos(p.position);
        let mut player = Player {
            alive: p.health > 0.0,
            creative: p.game_mode == 1,
            block_pos: here,
            xp_progress_points: 0,
            xp_level: 0,
        };
        let (target, valid) = {
            let world = Reader { host: &*host, level: Some(level), height };
            (world.elevator_at(to), logic::validate(&world, &player, from, to, &cfg))
        };
        let (Some(target), Ok(())) = (target, valid) else {
            return;
        };

        // XP: como no original, só se `useXP` estiver ligado e o jogador não for criativo.
        let mut cost = 0;
        if cfg.use_xp && !player.creative {
            let Some((xp_level, progress)) = Self::read_xp(host, &p) else {
                return;
            };
            player.xp_level = u32::try_from(xp_level).unwrap_or(0);
            player.xp_progress_points = logic::xp_progress_points(xp_level, progress);
            match logic::xp_cost(&player, &cfg) {
                Ok(c) => cost = c,
                Err(Reject::MissingXp) => {
                    host.action_bar(
                        uuid,
                        Text::translate_with_fallback(
                            "elevatorid.message.missing_xp",
                            "You do not have enough XP to use elevators",
                            &[],
                        )
                        .with_color(RED),
                    );
                    return;
                }
                Err(_) => return,
            }
        }
        if cost > 0 && !host.run_command(&format!("xp add {} -{} points", p.name, cost)) {
            return; // não conseguiu cobrar: não teleporta de graça
        }

        let dest = logic::destination(to, &target, p.position[0], p.position[2], &cfg);
        let yaw = match dest.yaw {
            Yaw::Keep => "~".to_string(),
            Yaw::Set(v) => format!("{v}"),
        };
        let pitch = match dest.pitch {
            Pitch::Keep => "~",
            Pitch::Reset => "0",
        };
        let line = format!(
            "execute in {} run tp {} {} {} {} {} {}",
            p.dimension, p.name, dest.x, dest.y, dest.z, yaw, pitch
        );
        if host.run_command(&line) {
            host.play_sound(
                TELEPORT_SOUND,
                [f64::from(to.x) + 0.5, f64::from(to.y) + 0.5, f64::from(to.z) + 0.5],
                4, // SoundSource.BLOCKS
                1.0,
                1.0,
            );
        }
    }
}

impl Hooks for Mod {
    /// Cliente: guarda o estado das teclas de pular e agachar.
    fn movement_input(&self, _host: &dyn Host, event: &mut MovementInputEvent) {
        self.jump.store(event.jump, Ordering::Relaxed);
        self.sneak.store(event.sneak, Ordering::Relaxed);
    }

    /// Cliente: no instante em que uma tecla é apertada, tenta usar o elevador.
    /// Como no original, agachar é visto antes de pular.
    fn tick(&self, host: &mut dyn Host, _event: &mut TickEvent) {
        if !host.window() {
            return;
        }
        let sneaking = self.sneak.load(Ordering::Relaxed);
        if self.last_sneak.swap(sneaking, Ordering::Relaxed) != sneaking && sneaking {
            self.client_try(host, Dir::Down);
        }
        let jumping = self.jump.load(Ordering::Relaxed);
        if self.last_jump.swap(jumping, Ordering::Relaxed) != jumping && jumping {
            self.client_try(host, Dir::Up);
        }
    }

    /// Servidor: com `mobSpawn = false` (o padrão do original), nenhum mob nasce em cima de um
    /// elevador. O original faz isso com `isValidSpawn`; aqui a lista de mobs possíveis daquela
    /// posição fica vazia quando o bloco de baixo é um elevador.
    fn potential_spawns(&self, host: &dyn Host, event: &mut PotentialSpawnsEvent<'_>) -> Flow {
        if self.config(host).mob_spawn {
            return Flow::Continue;
        }
        let [x, y, z] = event.pos;
        let on_elevator = host
            .level(event.dimension)
            .and_then(|level| host.level_block(level, [x, y - 1, z]))
            .is_some_and(|name| DyeColor::from_block_name(&name).is_some());
        if on_elevator { Flow::Cancel } else { Flow::Continue }
    }

    /// Servidor: o pedido de teleporte de um cliente.
    fn net_from_client(&self, host: &mut dyn Host, event: &mut NetFromClientEvent<'_>) {
        if event.channel != logic::CHANNEL {
            return;
        }
        if let Some((from, to)) = logic::decode_request(event.data) {
            self.server_teleport(host, event.player, from, to);
        }
    }
}
