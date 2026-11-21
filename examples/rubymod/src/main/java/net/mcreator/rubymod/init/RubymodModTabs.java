/*
 *    MCreator note: This file will be REGENERATED on each build.
 */

package net.mcreator.rubymod.init;

import net.neoforged.neoforge.registries.DeferredRegister;
import net.neoforged.neoforge.event.BuildCreativeModeTabContentsEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.bus.api.SubscribeEvent;

import net.minecraft.world.item.CreativeModeTabs;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.core.registries.Registries;

import net.mcreator.rubymod.RubymodMod;

@EventBusSubscriber
public class RubymodModTabs {
	public static final DeferredRegister<CreativeModeTab> REGISTRY = DeferredRegister.create(Registries.CREATIVE_MODE_TAB, RubymodMod.MODID);

	@SubscribeEvent public static void buildTabContentsVanilla(BuildCreativeModeTabContentsEvent tabData) {
		if (tabData.getTabKey() == CreativeModeTabs.INGREDIENTS) {
			tabData.accept(RubymodModItems.RUBY.get());
		}
	}
}