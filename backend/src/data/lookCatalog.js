export const lookCatalog = {
  trabalho: {
    formal: [
      {
        name: "Executive Precision",
        styleDescription: "Silhueta alinhada com cortes retos, ideal para rotinas corporativas e reuniões de alto impacto.",
        colorCombo: "Marinho + Branco + Grafite",
        accessoriesTip: "Relógio clássico e cinto de couro estruturado.",
        top: "Camisa de algodão premium",
        bottom: "Calça de alfaiataria slim",
        shoes: "Oxford ou loafer em couro",
        extraLayer: "Blazer estruturado",
        occasions: ["trabalho", "viagem", "encontro"],
        tags: ["formal", "minimalista", "executivo"]
      }
    ],
    criativo: [
      {
        name: "Studio Smart",
        styleDescription: "Combinação moderna para quem mistura produtividade com expressão pessoal.",
        colorCombo: "Off-white + Verde oliva + Preto",
        accessoriesTip: "Óculos de design e corrente discreta.",
        top: "Camiseta premium gola alta",
        bottom: "Calça reta cropped",
        shoes: "Tênis de couro minimalista",
        extraLayer: "Sobretudo leve",
        occasions: ["trabalho", "passeio", "encontro"],
        tags: ["criativo", "urbano", "moderno"]
      }
    ]
  },
  academia: {
    fitness: [
      {
        name: "Energy Flow",
        styleDescription: "Look funcional para treino e deslocamentos rápidos com estética esportiva premium.",
        colorCombo: "Preto + Cinza + Neon pontual",
        accessoriesTip: "Smartwatch e garrafa térmica estilizada.",
        top: "Camiseta dry-fit respirável",
        bottom: "Legging ou jogger técnico",
        shoes: "Tênis de performance",
        extraLayer: "Jaqueta corta-vento",
        occasions: ["academia", "casual", "viagem"],
        tags: ["fitness", "casual", "urbano"]
      }
    ],
    casual: [
      {
        name: "Athleisure Pulse",
        styleDescription: "Conforto e estética para rotina dinâmica entre compromissos e atividades físicas.",
        colorCombo: "Areia + Preto + Azul petróleo",
        accessoriesTip: "Boné clean e mochila compacta.",
        top: "Regata estruturada ou tee oversized",
        bottom: "Short esportivo premium",
        shoes: "Tênis lifestyle",
        extraLayer: "Hoodie leve",
        occasions: ["academia", "passeio", "casual"],
        tags: ["fitness", "casual", "universitário"]
      }
    ]
  },
  passeio: {
    universitario: [
      {
        name: "Campus Cool",
        styleDescription: "Confortável e estiloso para aulas, biblioteca e encontros rápidos.",
        colorCombo: "Azul denim + Branco + Bege",
        accessoriesTip: "Mochila funcional e fones sem fio.",
        top: "Camiseta básica premium",
        bottom: "Jeans reto",
        shoes: "Tênis casual",
        extraLayer: "Jaqueta jeans",
        occasions: ["passeio", "casual", "viagem"],
        tags: ["universitário", "casual", "urbano"]
      }
    ],
    minimalista: [
      {
        name: "Neutral Ease",
        styleDescription: "Look clean com poucos elementos e alto refinamento visual.",
        colorCombo: "Branco + Bege + Fendi",
        accessoriesTip: "Acessórios metal escovado em baixa quantidade.",
        top: "Camisa ampla em linho",
        bottom: "Calça reta em sarja",
        shoes: "Mule ou tênis monocromático",
        extraLayer: "Cardigan longo",
        occasions: ["passeio", "encontro", "casual"],
        tags: ["minimalista", "social", "criativo"]
      }
    ]
  },
  encontro: {
    social: [
      {
        name: "Evening Signature",
        styleDescription: "Sofisticação equilibrada para impressionar sem exageros.",
        colorCombo: "Preto + Vinho + Off-white",
        accessoriesTip: "Perfume marcante e joia minimal.",
        top: "Camisa texturizada",
        bottom: "Calça slim premium",
        shoes: "Bota Chelsea ou scarpin clássico",
        extraLayer: "Blazer slim ou trench curto",
        occasions: ["encontro", "festa", "trabalho"],
        tags: ["social", "formal", "criativo"]
      }
    ],
    criativo: [
      {
        name: "Artsy Date",
        styleDescription: "Peças modernas com toque autoral para ocasiões sociais e culturais.",
        colorCombo: "Terracota + Preto + Verde escuro",
        accessoriesTip: "Bolsa statement e anéis orgânicos.",
        top: "Blusa com recorte",
        bottom: "Saia midi ou calça ampla",
        shoes: "Tênis fashion ou loafer bold",
        extraLayer: "Jaqueta de couro leve",
        occasions: ["encontro", "passeio", "festa"],
        tags: ["criativo", "urbano", "social"]
      }
    ]
  },
  festa: {
    urbano: [
      {
        name: "Night Urban Lux",
        styleDescription: "Vibe noturna com atitude e contraste de texturas.",
        colorCombo: "Preto + Prata + Azul noturno",
        accessoriesTip: "Pulseiras metálicas e clutch compacta.",
        top: "Camisa preta acetinada",
        bottom: "Calça resinada",
        shoes: "Bota de couro",
        extraLayer: "Jaqueta bomber",
        occasions: ["festa", "encontro", "viagem"],
        tags: ["urbano", "criativo", "social"]
      }
    ]
  },
  viagem: {
    casual: [
      {
        name: "Transit Comfort",
        styleDescription: "Praticidade para deslocamento com acabamento elegante.",
        colorCombo: "Cinza + Azul claro + Branco",
        accessoriesTip: "Mala de mão smart e óculos UV.",
        top: "Moletom premium",
        bottom: "Calça jogger alfaiatada",
        shoes: "Slip-on ou tênis ultraleve",
        extraLayer: "Parka dobrável",
        occasions: ["viagem", "casual", "passeio"],
        tags: ["casual", "universitário", "minimalista"]
      }
    ]
  },
  casual: {
    urbano: [
      {
        name: "City Everyday",
        styleDescription: "Visual urbano refinado para rotina multitarefa.",
        colorCombo: "Chumbo + Branco + Caramelo",
        accessoriesTip: "Bolsa transversal e relógio esportivo.",
        top: "Polo de malha",
        bottom: "Calça cargo slim",
        shoes: "Tênis retro",
        extraLayer: "Jaqueta utilitária",
        occasions: ["casual", "passeio", "trabalho"],
        tags: ["urbano", "casual", "fitness"]
      }
    ]
  }
};

export const priceBands = {
  barato: {
    range: "R$120 - R$280",
    strategy: "Foco em custo-benefício com peças versáteis e básicas inteligentes."
  },
  meio_termo: {
    range: "R$300 - R$650",
    strategy: "Equilíbrio entre qualidade e design com materiais superiores."
  },
  caro: {
    range: "R$700 - R$1800+",
    strategy: "Acabamento premium, corte impecável e peças assinatura."
  }
};
