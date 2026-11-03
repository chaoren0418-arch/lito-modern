package local.lito381.server;

import java.util.List;

/**
 * Exact direct effects contributed by equipped LiTo rental objects whose
 * selected-3.80 templates are pinned by {@link RentalItemDefinitionCatalog}.
 *
 * <p>Source-bound legacy equipment is deliberately excluded: its effects are
 * already present in {@link SourceEquipmentMagicStats}.  This record therefore
 * represents only the session-owned delta and can be safely layered on top of
 * the selected character snapshot.</p>
 */
record RentalEquipmentBonuses(
        int matchedItems, int matchedWeapons, int matchedArmors,
        int strength, int constitution, int dexterity,
        int intelligence, int wisdom, int charisma,
        int maximumHp, int maximumMp, int hpRegen, int mpRegen,
        int spellPower, int magicResistance, boolean haste, int magicDamage,
        int damageReduction, int meleeReduction, int rangedReduction,
        int weightReduction,
        int meleeHit, int meleeDamage, int rangedHit, int rangedDamage,
        int waterResistance, int windResistance,
        int fireResistance, int earthResistance,
        int stunResistance, int stoneResistance, int sleepResistance,
        int freezeResistance, int sustainResistance, int blindResistance) {

    static RentalEquipmentBonuses from(List<RuntimeInventory.Item> items) {
        if (items == null) throw new IllegalArgumentException("inventory items are required");
        int matchedItems = 0;
        int matchedWeapons = 0;
        int matchedArmors = 0;
        int strength = 0;
        int constitution = 0;
        int dexterity = 0;
        int intelligence = 0;
        int wisdom = 0;
        int charisma = 0;
        int maximumHp = 0;
        int maximumMp = 0;
        int hpRegen = 0;
        int mpRegen = 0;
        int spellPower = 0;
        int magicResistance = 0;
        boolean haste = false;
        int magicDamage = 0;
        int damageReduction = 0;
        int meleeReduction = 0;
        int rangedReduction = 0;
        int weightReduction = 0;
        int meleeHit = 0;
        int meleeDamage = 0;
        int rangedHit = 0;
        int rangedDamage = 0;
        int waterResistance = 0;
        int windResistance = 0;
        int fireResistance = 0;
        int earthResistance = 0;
        int stunResistance = 0;
        int stoneResistance = 0;
        int sleepResistance = 0;
        int freezeResistance = 0;
        int sustainResistance = 0;
        int blindResistance = 0;

        for (RuntimeInventory.Item item : items) {
            if (item == null || !item.equipped() || item.sourceBound()) continue;
            BonusSource source = source(Math.toIntExact(item.definition().catalogId()));
            if (source == null) continue;
            matchedItems = Math.addExact(matchedItems, 1);
            if ("weapon".equals(source.kind())) matchedWeapons++;
            else matchedArmors++;

            strength = add(strength, source, "add_str");
            constitution = add(constitution, source, "add_con");
            dexterity = add(dexterity, source, "add_dex");
            intelligence = add(intelligence, source, "add_int");
            wisdom = add(wisdom, source, "add_wis");
            charisma = add(charisma, source, "add_cha");
            maximumHp = add(maximumHp, source, "add_hp");
            maximumMp = add(maximumMp, source, "add_mp");
            hpRegen = add(hpRegen, source, "add_hpr");
            mpRegen = add(mpRegen, source, "add_mpr");
            spellPower = add(spellPower, source, "add_sp");
            magicResistance = add(magicResistance, source, "m_def");
            magicDamage = add(magicDamage, source, "magicdmgmodifier");
            haste |= source.integer("haste_item") == 1;

            if ("armor".equals(source.kind())) {
                damageReduction = add(damageReduction, source, "damage_reduction");
                meleeReduction = add(meleeReduction, source, "add_melee_redux");
                rangedReduction = add(rangedReduction, source, "add_bow_redux");
                weightReduction = add(weightReduction, source, "weight_reduction");
                meleeHit = add(meleeHit, source, "hit_modifier");
                meleeDamage = add(meleeDamage, source, "dmg_modifier");
                rangedHit = add(rangedHit, source, "bow_hit_modifier");
                rangedDamage = add(rangedDamage, source, "bow_dmg_modifier");
                waterResistance = add(waterResistance, source, "defense_water");
                windResistance = add(windResistance, source, "defense_wind");
                fireResistance = add(fireResistance, source, "defense_fire");
                earthResistance = add(earthResistance, source, "defense_earth");
                stunResistance = add(stunResistance, source, "regist_stun");
                stoneResistance = add(stoneResistance, source, "regist_stone");
                sleepResistance = add(sleepResistance, source, "regist_sleep");
                freezeResistance = add(freezeResistance, source, "regist_freeze");
                sustainResistance = add(sustainResistance, source, "regist_sustain");
                blindResistance = add(blindResistance, source, "regist_blind");
            }
        }
        return new RentalEquipmentBonuses(
                matchedItems, matchedWeapons, matchedArmors,
                strength, constitution, dexterity, intelligence, wisdom, charisma,
                maximumHp, maximumMp, hpRegen, mpRegen, spellPower,
                magicResistance, haste, magicDamage, damageReduction,
                meleeReduction, rangedReduction, weightReduction,
                meleeHit, meleeDamage, rangedHit, rangedDamage,
                waterResistance, windResistance, fireResistance, earthResistance,
                stunResistance, stoneResistance, sleepResistance,
                freezeResistance, sustainResistance, blindResistance);
    }

    static RentalEquipmentBonuses none() {
        return from(List.of());
    }

    private static int add(int current,
                           BonusSource source,
                           String field) {
        return Math.addExact(current, source.integer(field));
    }
    private interface BonusSource {
        String kind();
        int integer(String field);
    }

    /** Preserve selected-3.80 precedence; merchant definitions retain their own provenance. */
    private static BonusSource source(int catalogId) {
        RentalItemDefinitionCatalog.SourceTemplate selected = RentalItemDefinitionCatalog.source(catalogId);
        if (selected != null) {
            return new BonusSource() {
                public String kind() { return selected.kind(); }
                public int integer(String field) { return selected.integer(field); }
            };
        }
        NativeLitoRentalMerchantCatalog.Item merchant = NativeLitoRentalMerchantCatalog.findBySourceItemId(catalogId);
        if (merchant == null) return null;
        // Already decoded from the hash-pinned retained merchant TSV. No fallback database.
        InventoryCatalog.NativeArmorStatus armor = merchant.definition().sourceArmorStatus();
        return new BonusSource() {
            public String kind() { return armor == null ? "weapon" : "armor"; }
            public int integer(String field) {
                return switch (field) {
                    case "add_str" -> armor == null ? 0 : armor.strength();
                    case "add_con" -> armor == null ? 0 : armor.constitution();
                    case "add_dex" -> armor == null ? 0 : armor.dexterity();
                    case "add_int" -> armor == null ? 0 : armor.intelligence();
                    case "add_wis" -> armor == null ? 0 : armor.wisdom();
                    case "add_cha" -> armor == null ? 0 : armor.charisma();
                    case "add_hp" -> armor == null ? 0 : armor.hitPoints();
                    case "add_mp" -> armor == null ? 0 : armor.manaPoints();
                    case "add_sp" -> armor == null ? 0 : armor.spellPower();
                    case "m_def" -> armor == null ? 0 : armor.magicResistance();
                    case "defense_water" -> armor == null ? 0 : armor.waterResistance();
                    case "defense_wind" -> armor == null ? 0 : armor.windResistance();
                    case "defense_fire" -> armor == null ? 0 : armor.fireResistance();
                    case "defense_earth" -> armor == null ? 0 : armor.earthResistance();
                    case "regist_stun" -> armor == null ? 0 : armor.stunResistance();
                    case "regist_stone" -> armor == null ? 0 : armor.stoneResistance();
                    case "regist_sleep" -> armor == null ? 0 : armor.sleepResistance();
                    case "regist_freeze" -> armor == null ? 0 : armor.freezeResistance();
                    case "regist_sustain" -> armor == null ? 0 : armor.sustainResistance();
                    case "regist_blind" -> armor == null ? 0 : armor.blindResistance();
                    case "haste_item" -> armor != null && armor.haste() ? 1 : 0;
                    // These effects are absent from the retained merchant source schema.
                    // Weapon damage/hit and native AC remain in their existing evaluators.
                    case "add_hpr", "add_mpr", "magicdmgmodifier", "damage_reduction",
                         "add_melee_redux", "add_bow_redux", "weight_reduction",
                         "hit_modifier", "dmg_modifier", "bow_hit_modifier", "bow_dmg_modifier" -> 0;
                    default -> throw new IllegalArgumentException("unknown merchant bonus field " + field);
                };
            }
        };
    }

}
