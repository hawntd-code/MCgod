/*
 *    MCreator note: This file will be REGENERATED on each build.
 */

package net.mcreator.rubymod.init;

import net.neoforged.neoforge.registries.DeferredRegister;
import net.neoforged.neoforge.registries.DeferredItem;

import net.minecraft.world.item.Item;

import net.mcreator.rubymod.item.RubyItem;
import net.mcreator.rubymod.RubymodMod;

public class RubymodModItems {
	public static final DeferredRegister.Items REGISTRY = DeferredRegister.createItems(RubymodMod.MODID);

	public static final DeferredItem<Item> RUBY;

	static {
		RUBY = REGISTRY.register("ruby", RubyItem::new);
	}

	// Start of user code block custom items
	// End of user code block custom items
}