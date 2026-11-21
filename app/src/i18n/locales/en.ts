import type { Messages } from "./es";

/** English UI strings. Must have exactly the same keys as `es.ts`. */
export const en: Messages = {
  language: {
    label: "Language",
    es: "Español",
    en: "English",
  },
  topBar: {
    noProject: "No project open",
    projectsLater: "Projects arrive in phase 1",
    needProject: "Open or create a project first",
    build: "Build",
    play: "Test in Minecraft",
    show: "Show",
    hide: "Hide",
    panels: {
      sidebar: "elements panel",
      console: "console",
      ai: "AI assistant",
    },
  },
  activities: {
    nav: "Sections",
    elements: "Mod elements",
    resources: "Textures, sounds and models",
    variables: "Variables",
    search: "Search the project",
    knowledge: "Knowledge base",
    settings: "Settings",
  },
  sidebar: {
    filter: "Filter elements",
    elements: {
      title: "Elements",
      emptyTitle: "No project open",
      emptyText:
        "Your mod's items, blocks, creatures, dimensions and procedures will show up here, grouped by type with a preview.",
      phase: "Opening MCreator workspaces arrives in phase 1.",
    },
    resources: {
      title: "Resources",
      emptyTitle: "No resources yet",
      emptyText:
        "The project's textures, sounds, 3D models and structures. You'll paint, import and preview them without leaving the app.",
      phase: "The texture editor arrives in phase 2.",
    },
    variables: {
      title: "Variables",
      emptyTitle: "No variables",
      emptyText: "Global, world and player variables your procedures use to keep data between sessions.",
      phase: "They arrive with procedures in phase 3.",
    },
    search: {
      title: "Search",
      emptyTitle: "Nothing to search yet",
      emptyText: "Find elements, fields, broken references and text across the whole project.",
      phase: "Project-wide search arrives in phase 5.",
    },
    knowledge: {
      title: "Knowledge base",
      emptyTitle: "What each option does",
      emptyText: "Modding guides and an explanation of every field in every editor. The same information the AI reads.",
      phase: "Arrives in phase 6, starting from MCreator's 729 help texts.",
    },
  },
  welcome: {
    tab: "Home",
    title: "Make your first mod.",
    intro:
      "Items, blocks, creatures and dimensions for Minecraft. Make them with the editors or ask the AI in the right panel, then review every change it makes.",
    requirements: "Requirements",
    checks: {
      ok: "Ready",
      pending: "Checking",
      warn: "Needs attention",
      engine: "Rust engine",
      connecting: "Connecting…",
      searching: "Searching…",
      engineOffline: "This view is running in a browser. Open the desktop app to connect it.",
      javaOffline: "Checked from the desktop app.",
      engineError: (message: string) => `No response: ${message}`,
      engineReady: (version: string, os: string) => `Connected · version ${version} · ${os}`,
      java: (required: number) => `Java ${required} or newer`,
      javaMissing: (required: number) => `Not found. Install a JDK ${required} to build mods.`,
      javaOld: (version: string, required: number) => `You have Java ${version}; NeoForge mods need Java ${required}.`,
    },
    roadmap: "Roadmap",
    roadmapIntro: "Current phase: 0, the app's foundation. Each phase ends with a feature you can already use.",
    phases: [
      { name: "Compatible core", text: "Open MCreator workspaces, build the mod and launch it in Minecraft." },
      { name: "Live AI", text: "Connect your provider and watch the AI create items, blocks and recipes." },
      { name: "Procedures", text: "All 528 logic blocks, with Blockly." },
      { name: "MCreator parity", text: "All 37 element types, Data Pack, Resource Pack and Bedrock." },
      { name: "Editors and viewers", text: "3D models, animations, GUIs and a code editor." },
      { name: "Knowledge", text: "Help on every option and modding guides." },
    ],
  },
  ai: {
    title: "Assistant",
    noProvider: "No provider",
    connectTitle: "Connect your AI",
    connectText:
      "Bring your own key. The AI uses the same tools you do, and each change it makes shows up in the project as it happens.",
    providersLabel: "AI provider",
    providers: {
      anthropic: "Claude models · API key",
      openai: "GPT models · API key",
      gemini: "Google AI Studio API key",
      deepseek: "DeepSeek models · API key",
      ollama: "Local models on your PC · no key",
      compatible: "OpenRouter, LM Studio and others",
    },
    compatibleName: "OpenAI-compatible",
    pickedNote:
      "Provider connections arrive in phase 2. The key will be kept in the Windows credential store, never inside the project.",
    abilitiesTitle: "What it will do",
    abilities: [
      "Create and edit mod elements",
      "Build and launch Minecraft to test",
      "Read the console and fix errors",
      "Look things up in the knowledge base",
    ],
    inputLabel: "Message for the AI",
    inputPlaceholder: "Connect a provider to message the AI",
    send: "Send",
  },
  console: {
    gradle: "Gradle console",
    gradleEmpty: "No builds yet. When you build or launch Minecraft, the output shows up here as it happens.",
    ai: "AI activity",
    aiEmpty: "A log of every tool the AI uses will appear here: what changed, in which element and with what result.",
  },
  status: {
    noProject: "No project",
    generator: "Generator: none",
    mcpOff: "MCP off",
    javaUnknown: "Java: —",
    javaMissing: "Java not found",
    java: (version: string) => `Java ${version}`,
  },
};
