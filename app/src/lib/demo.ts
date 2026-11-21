/**
 * Proyecto de ejemplo para la vista en el navegador (`?demo`) y las pruebas de interfaz.
 * Refleja el workspace de prueba del motor: crates/mcgod-workspace/tests/fixtures/rubymod.
 * No lo generó MCreator; está escrito a mano siguiendo su formato.
 */
import type { ElementDefinition, WorkspaceSummary } from "./engine";

export const DEMO_FILE = "demo://rubymod/rubymod.mcreator";

export const demoSummary: WorkspaceSummary = {
  root: "demo://rubymod",
  file: DEMO_FILE,
  modid: "rubymod",
  name: "Ruby Mod",
  version: "1.0.0",
  author: "MCgod",
  description: "Workspace de ejemplo para las pruebas de MCgod",
  generator: { id: "neoforge-1.21.1", flavor: "neoforge", minecraft: "1.21.1" },
  mcreatorVersion: 202500315219,
  elements: [
    { name: "Ruby", type: "item", registryName: "ruby", folder: "~/Rubí", compiles: true, lockedCode: false, hasDefinition: true },
    { name: "RubyBlock", type: "block", registryName: "ruby_block", folder: "~/Rubí", compiles: true, lockedCode: false, hasDefinition: true },
    { name: "RubyOre", type: "block", registryName: "ruby_ore", folder: "~/Rubí", compiles: true, lockedCode: false, hasDefinition: true },
    { name: "RubyPickaxe", type: "tool", registryName: "ruby_pickaxe", folder: null, compiles: false, lockedCode: false, hasDefinition: false },
    { name: "RubyBlockRecipe", type: "recipe", registryName: "ruby_block_recipe", folder: null, compiles: true, lockedCode: false, hasDefinition: true },
    { name: "GlowOnStep", type: "procedure", registryName: "glow_on_step", folder: null, compiles: true, lockedCode: true, hasDefinition: true },
    { name: "RubyUtils", type: "code", registryName: "ruby_utils", folder: null, compiles: true, lockedCode: true, hasDefinition: true },
  ],
  variables: 1,
  sounds: 0,
  tags: 1,
  languages: ["en_us", "es_mx"],
};

export const demoDefinitions: Record<string, ElementDefinition> = {
  Ruby: {
    _fv: 92,
    _type: "item",
    definition: {
      name: "Ruby",
      rarity: "UNCOMMON",
      creativeTabs: [{ value: "INGREDIENTS" }],
      stackSize: 64,
      enchantability: 0,
      useDuration: 0,
      toolType: 1.0,
      damageCount: 0,
      destroyAnyBlock: false,
      immuneToFire: true,
      isPiglinCurrency: false,
      enableMeleeDamage: false,
      damageVsEntity: 4.0,
      attackSpeed: 1.2,
      inventorySize: 9,
      inventoryStackSize: 99,
      texture: { texture: "ruby" },
    },
  },
  RubyBlock: {
    _fv: 92,
    _type: "block",
    definition: {
      name: "Block of Ruby",
      hardness: 5.0,
      resistance: 6.0,
      soundOnStep: { value: "METAL" },
      luminance: 0,
      requiresCorrectTool: true,
      destroyTool: "pickaxe",
      vanillaToolTier: "IRON",
      creativeTabs: [{ value: "BUILDING_BLOCKS" }],
    },
  },
  RubyOre: {
    _fv: 92,
    _type: "block",
    definition: {
      name: "Ruby Ore",
      hardness: 3.0,
      resistance: 3.0,
      soundOnStep: { value: "STONE" },
      luminance: 0,
      requiresCorrectTool: true,
      destroyTool: "pickaxe",
      vanillaToolTier: "IRON",
      generateFeature: true,
      frequencyPerChunks: 4,
      frequencyOnChunk: 6,
      minGenerateHeight: -48,
      maxGenerateHeight: 16,
    },
  },
  RubyBlockRecipe: {
    _fv: 92,
    _type: "recipe",
    definition: {
      recipeType: "Crafting",
      recipeShapeless: false,
      recipeRetstackSize: 1,
      group: "",
      cookingBookCategory: "MISC",
      craftingBookCategory: "BUILDING",
      recipeSlots: Array.from({ length: 9 }, () => ({ value: "CUSTOM:Ruby" })),
      recipeReturnStack: { value: "CUSTOM:RubyBlock" },
    },
  },
  GlowOnStep: {
    _fv: 92,
    _type: "procedure",
    definition: {
      procedurexml:
        '<xml xmlns="https://developers.google.com/blockly/xml"><block type="event_trigger" deletable="false" x="40" y="40"><field name="trigger">player_walks_on_block</field></block></xml>',
    },
  },
};
