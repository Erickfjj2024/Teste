//! Elevator para o Horizon: pular sobe para o elevador de cima, agachar desce para o de baixo.
//!
//! Port do ElevatorMod (VsnGamer, MIT; veja NOTICE.md). A lógica pura está em `../../src/lib.rs`
//! (testada sem o jogo); este arquivo só liga essa lógica aos ganchos do servidor.
//!
//! Como funciona: o servidor avisa quando um jogador pula (`player_jump`) ou agacha
//! (`player_toggle_sneak`). O mod guarda o pedido. No fim do tick (`server_tick`) ele executa o
//! `/tp`, porque dentro do gancho de um jogador o jogo não deixa rodar comandos.
use std::sync::Mutex;

use horizon_client::mod_api::{
    Hooks, Host, PlayerJumpEvent, PlayerToggleSneakEvent, ServerTickEvent, ServerTickPhase,
};

// Parte da lógica (elevador direcional, limites de configuração) ainda não é usada pelo jogo.
#[allow(dead_code)]
#[path = "../../src/lib.rs"]
mod logic;

use logic::{BlockPos, Config, Destination, Dir, DyeColor, Elevator, Facing, Pitch, Player, Yaw, World};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod::default())
}

/// Som tocado no destino. O ElevatorMod usa o próprio `elevatorid:teleport`; até o `.hmod` trazer
/// esse som, usa-se um som do jogo.
const TELEPORT_SOUND: &str = "minecraft:block.piston.extend";

struct Pending {
    name: String,
    dimension: String,
    to: BlockPos,
    dest: Destination,
}

#[derive(Default)]
struct Mod {
    pending: Mutex<Vec<Pending>>,
}

/// O mundo de um servidor, visto pelo `Host`.
struct HostWorld<'a> {
    host: &'a dyn Host,
    level: horizon_client::mod_api::LevelRef,
}

fn arr(p: BlockPos) -> [i32; 3] {
    [p.x, p.y, p.z]
}

impl World for HostWorld<'_> {
    fn elevator_at(&self, pos: BlockPos) -> Option<Elevator> {
        let name = self.host.level_block(self.level, arr(pos))?;
        let color = DyeColor::from_block_name(&name)?;
        // Altura da forma de colisão do bloco, para o jogador ficar em cima dele.
        let support_height = self
            .host
            .block_collision(arr(pos))
            .iter()
            .map(|b| b[4])
            .fold(0.0_f64, f64::max);
        Some(Elevator {
            color,
            // O estado direcional e a camuflagem dependem de uma entidade de bloco, ainda não portada.
            directional: false,
            facing: Facing::South,
            support_height: if support_height > 0.0 { support_height } else { 1.0 },
        })
    }

    fn is_suffocating(&self, pos: BlockPos) -> bool {
        // Sufoca quem está dentro: um bloco com colisão de cubo inteiro.
        self.host.block_collision(arr(pos)).iter().any(|b| {
            b[3] - b[0] >= 0.999 && b[4] - b[1] >= 0.999 && b[5] - b[2] >= 0.999
        })
    }

    fn in_build_height(&self, y: i32) -> bool {
        self.host
            .level_height(self.level)
            .is_some_and(|(min, height)| y >= min && y < min + height)
    }

    fn is_loaded(&self, pos: BlockPos) -> bool {
        self.host.level_is_loaded(self.level, arr(pos))
    }
}

/// Nomes de jogador válidos: letras, números e `_`. Barra qualquer coisa que possa virar outro comando.
fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 16 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Mod {
    fn request(&self, host: &mut dyn Host, uuid: u128, dir: Dir) {
        let cfg = Config::default();
        let Some(p) = host.server_players().into_iter().find(|p| p.uuid == uuid) else {
            return;
        };
        // Espectador não usa elevador; jogador morto também não.
        if p.game_mode == 3 || p.health <= 0.0 || !safe_name(&p.name) {
            return;
        }
        // Os dados de bloco e colisão do `Host` valem para o mundo em que o gancho roda.
        if host.dimension() != p.dimension {
            return;
        }
        let Some(level) = host.level(&p.dimension) else {
            return;
        };
        let world = HostWorld { host: &*host, level };

        let here = BlockPos::new(
            p.position[0].floor() as i32,
            p.position[1].floor() as i32,
            p.position[2].floor() as i32,
        );
        let Some(from) = logic::origin_elevator(&world, here, &cfg) else {
            return;
        };
        let Some(to) = logic::find_target(&world, from, dir, &cfg) else {
            return;
        };
        let player = Player {
            alive: true,
            creative: p.game_mode == 1,
            block_pos: here,
            // O gancho não vê o XP do jogador; por isso `use_xp` fica desligado (padrão do original).
            xp_progress_points: 0,
            xp_level: 0,
        };
        if logic::validate(&world, &player, from, to, &cfg).is_err() || logic::xp_cost(&player, &cfg).is_err() {
            return;
        }
        let Some(target) = world.elevator_at(to) else {
            return;
        };
        let dest = logic::destination(to, &target, p.position[0], p.position[2], &cfg);
        self.pending.lock().unwrap().push(Pending { name: p.name, dimension: p.dimension, to, dest });
    }
}

impl Hooks for Mod {
    /// Pular: sobe para o próximo elevador acima.
    fn player_jump(&self, host: &mut dyn Host, event: &mut PlayerJumpEvent<'_>) {
        self.request(host, event.player, Dir::Up);
    }

    /// Agachar: desce para o próximo elevador abaixo.
    fn player_toggle_sneak(&self, host: &mut dyn Host, event: &mut PlayerToggleSneakEvent<'_>) {
        if event.sneaking {
            self.request(host, event.player, Dir::Down);
        }
    }

    /// No fim do tick: executa os teleportes pedidos.
    fn server_tick(&self, host: &mut dyn Host, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { .. }) {
            return;
        }
        let todo = std::mem::take(&mut *self.pending.lock().unwrap());
        for p in todo {
            let yaw = match p.dest.yaw {
                Yaw::Keep => "~".to_string(),
                Yaw::Set(v) => format!("{v}"),
            };
            let pitch = match p.dest.pitch {
                Pitch::Keep => "~",
                Pitch::Reset => "0",
            };
            let line = format!(
                "execute in {} run tp {} {} {} {} {} {}",
                p.dimension, p.name, p.dest.x, p.dest.y, p.dest.z, yaw, pitch
            );
            if host.run_command(&line) {
                host.play_sound(
                    TELEPORT_SOUND,
                    [f64::from(p.to.x) + 0.5, f64::from(p.to.y) + 0.5, f64::from(p.to.z) + 0.5],
                    4, // SoundSource.BLOCKS
                    1.0,
                    1.0,
                );
            }
        }
    }
}
