/** Textos de la interfaz en español. Es la referencia: `en.ts` debe tener las mismas claves. */
export const es = {
  language: {
    label: "Idioma",
    es: "Español",
    en: "English",
  },
  topBar: {
    noProject: "Sin proyecto abierto",
    projectsLater: "Los proyectos llegan en la fase 1",
    needProject: "Abre o crea un proyecto primero",
    build: "Compilar",
    play: "Probar en Minecraft",
    show: "Mostrar",
    hide: "Ocultar",
    panels: {
      sidebar: "panel de elementos",
      console: "consola",
      ai: "asistente de IA",
    },
  },
  activities: {
    nav: "Secciones",
    elements: "Elementos del mod",
    resources: "Texturas, sonidos y modelos",
    variables: "Variables",
    search: "Buscar en el proyecto",
    knowledge: "Base de conocimiento",
    settings: "Ajustes",
  },
  sidebar: {
    filter: "Filtrar elementos",
    elements: {
      title: "Elementos",
      emptyTitle: "Ningún proyecto abierto",
      emptyText:
        "Aquí aparecerán los ítems, bloques, criaturas, dimensiones y procedures de tu mod, agrupados por tipo y con su vista previa.",
      phase: "Abrir workspaces de MCreator llega en la fase 1.",
    },
    resources: {
      title: "Recursos",
      emptyTitle: "Sin recursos todavía",
      emptyText:
        "Texturas, sonidos, modelos 3D y estructuras del proyecto. Podrás pintarlos, importarlos y verlos sin salir de la app.",
      phase: "El editor de texturas llega en la fase 2.",
    },
    variables: {
      title: "Variables",
      emptyTitle: "Sin variables",
      emptyText: "Las variables globales, de mundo y de jugador que usan tus procedures para guardar datos entre partidas.",
      phase: "Llegan con los procedures en la fase 3.",
    },
    search: {
      title: "Buscar",
      emptyTitle: "Nada que buscar aún",
      emptyText: "Busca elementos, campos, referencias rotas y texto dentro de todo el proyecto.",
      phase: "La búsqueda global llega en la fase 5.",
    },
    knowledge: {
      title: "Base de conocimiento",
      emptyTitle: "Qué hace cada opción",
      emptyText:
        "Guías de modding y la explicación de cada campo de cada editor. La misma información que consulta la IA.",
      phase: "Llega en la fase 6, con los 729 textos de ayuda de MCreator como punto de partida.",
    },
  },
  welcome: {
    tab: "Inicio",
    title: "Crea tu primer mod.",
    intro:
      "Ítems, bloques, criaturas y dimensiones para Minecraft. Hazlos con los editores o pídeselos a la IA del panel derecho y revisa cada cambio que hace.",
    requirements: "Requisitos",
    checks: {
      ok: "Listo",
      pending: "Comprobando",
      warn: "Atención",
      engine: "Motor en Rust",
      connecting: "Conectando…",
      searching: "Buscando…",
      engineOffline: "Esta vista corre en el navegador. Abre la app de escritorio para conectarlo.",
      javaOffline: "Se comprueba desde la app de escritorio.",
      engineError: (message: string) => `No respondió: ${message}`,
      engineReady: (version: string, os: string) => `Conectado · versión ${version} · ${os}`,
      java: (required: number) => `Java ${required} o superior`,
      javaMissing: (required: number) => `No encontrado. Instala un JDK ${required} para compilar mods.`,
      javaOld: (version: string, required: number) =>
        `Tienes Java ${version}; los mods de NeoForge necesitan Java ${required}.`,
    },
    roadmap: "Hoja de ruta",
    roadmapIntro: "Fase actual: 0, la base de la app. Cada fase termina con una función que ya se puede usar.",
    phases: [
      { name: "Núcleo compatible", text: "Abrir workspaces de MCreator, compilar el mod y abrirlo en Minecraft." },
      { name: "IA en vivo", text: "Conectar tu proveedor y ver a la IA crear ítems, bloques y recetas." },
      { name: "Procedures", text: "Los 528 bloques de lógica, con Blockly." },
      { name: "Paridad con MCreator", text: "Los 37 tipos de elemento, Data Pack, Resource Pack y Bedrock." },
      { name: "Editores y visores", text: "Modelos 3D, animaciones, GUIs y editor de código." },
      { name: "Conocimiento", text: "Ayuda en cada opción y guías de modding." },
    ],
  },
  ai: {
    title: "Asistente",
    noProvider: "Sin proveedor",
    connectTitle: "Conecta tu IA",
    connectText:
      "Usa tu propia clave. La IA trabaja con las mismas herramientas que tú y cada cambio que hace aparece en el proyecto en el momento.",
    providersLabel: "Proveedor de IA",
    providers: {
      anthropic: "Modelos Claude · clave de API",
      openai: "Modelos GPT · clave de API",
      gemini: "Clave de API de Google AI Studio",
      deepseek: "Modelos DeepSeek · clave de API",
      ollama: "Modelos locales en tu PC · sin clave",
      compatible: "OpenRouter, LM Studio y otros",
    },
    compatibleName: "Compatible con OpenAI",
    pickedNote:
      "La conexión con proveedores llega en la fase 2. La clave se guardará en el almacén de credenciales de Windows, nunca dentro del proyecto.",
    abilitiesTitle: "Qué podrá hacer",
    abilities: [
      "Crear y editar elementos del mod",
      "Compilar y abrir Minecraft para probar",
      "Leer la consola y corregir errores",
      "Consultar la base de conocimiento",
    ],
    inputLabel: "Mensaje para la IA",
    inputPlaceholder: "Conecta un proveedor para escribirle a la IA",
    send: "Enviar",
  },
  console: {
    gradle: "Consola de Gradle",
    gradleEmpty: "Sin compilaciones todavía. Cuando compiles o abras Minecraft, la salida aparecerá aquí mientras ocurre.",
    ai: "Actividad de la IA",
    aiEmpty: "Aquí quedará el registro de cada herramienta que use la IA: qué cambió, en qué elemento y con qué resultado.",
  },
  status: {
    noProject: "Sin proyecto",
    generator: "Generador: ninguno",
    mcpOff: "MCP apagado",
    javaUnknown: "Java: —",
    javaMissing: "Java no encontrado",
    java: (version: string) => `Java ${version}`,
  },
};

export type Messages = typeof es;
