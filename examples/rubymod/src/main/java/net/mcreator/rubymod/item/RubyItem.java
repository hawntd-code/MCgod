package net.mcreator.rubymod.item;

import net.minecraft.world.item.Rarity;
import net.minecraft.world.item.Item;

public class RubyItem extends Item {
	public RubyItem() {
		super(new Item.Properties().fireResistant().rarity(Rarity.UNCOMMON));
	}
}