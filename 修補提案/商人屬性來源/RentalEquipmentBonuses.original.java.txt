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
            RentalItemDefinitionCatalog.SourceTemplate source =
                    RentalItemDefinitionCatalog.source(
                            Math.toIntExact(item.definition().catalogId()));
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
                           RentalItemDefinitionCatalog.SourceTemplate source,
                           String field) {
        return Math.addExact(current, source.integer(field));
    }
}
