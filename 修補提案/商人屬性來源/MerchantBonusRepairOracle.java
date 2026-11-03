package local.lito381.server;

import java.lang.reflect.RecordComponent;
import java.util.*;

/** Detached immutable inputs only. Does not create sessions, databases or timers. */
public final class MerchantBonusRepairOracle {
    static String bonus(List<RuntimeInventory.Item> items) throws Exception {
        RentalEquipmentBonuses result = RentalEquipmentBonuses.from(items);
        StringJoiner values = new StringJoiner(",", "{", "}");
        for (RecordComponent field : RentalEquipmentBonuses.class.getRecordComponents()) {
            values.add("\"" + field.getName() + "\":" + field.getAccessor().invoke(result));
        }
        return values.toString();
    }

    static RuntimeInventory.Item item(InventoryCatalog.Definition definition, boolean equipped, boolean bound) {
        return new RuntimeInventory.Item(1L, definition, 1, equipped, 0, 0, true,
                RuntimeInventory.BLESSING_NORMAL, bound);
    }

    public static void main(String[] args) throws Exception {
        List<InventoryCatalog.Definition> definitions = new ArrayList<>();
        for (var source : RentalItemDefinitionCatalog.all()) definitions.add(source.definition());
        for (var merchant : NativeLitoRentalMerchantCatalog.all()) definitions.add(merchant.definition());
        definitions.sort(Comparator.comparingLong(InventoryCatalog.Definition::catalogId));
        StringJoiner rows = new StringJoiner(",");
        List<RuntimeInventory.Item> aggregate = new ArrayList<>();
        for (var definition : definitions) {
            RuntimeInventory.Item equipped = item(definition, true, false);
            aggregate.add(equipped);
            rows.add("{\"catalog_id\":" + definition.catalogId()
                    + ",\"selected_source\":" + (RentalItemDefinitionCatalog.source(Math.toIntExact(definition.catalogId())) != null)
                    + ",\"equipped\":" + bonus(Arrays.asList(null, equipped))
                    + ",\"unworn\":" + bonus(List.of(item(definition, false, false)))
                    + ",\"source_bound\":" + bonus(List.of(item(definition, true, true))) + "}");
        }
        boolean nullRejected = false;
        try { RentalEquipmentBonuses.from(null); } catch (IllegalArgumentException expected) { nullRejected = true; }
        System.out.println("{\"detached_only\":true,\"null_inventory_rejected\":" + nullRejected
                + ",\"items\":[" + rows + "],\"aggregate\":" + bonus(aggregate)
                + ",\"empty\":" + bonus(List.of()) + "}");
    }
}
