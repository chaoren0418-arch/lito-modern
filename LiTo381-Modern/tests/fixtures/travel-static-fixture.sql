CREATE DATABASE travel_static CHARACTER SET utf8;
USE travel_static;
CREATE TABLE `etcitem`  (
  `note` varchar(255) CHARACTER SET utf8 COLLATE utf8_general_ci NULL DEFAULT NULL COMMENT '企劃／維運備註（程式不讀取）',
  `item_id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '道具編號（主鍵）',
  `name` varchar(45) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL DEFAULT '' COMMENT '遊戲內顯示名稱',
  `classname` varchar(100) CHARACTER SET utf8 COLLATE utf8_general_ci NULL DEFAULT '0' COMMENT 'ItemExecutor 類別；0=無自訂類別',
  `name_id` varchar(45) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL DEFAULT '' COMMENT '客戶端字串資源 ID',
  `item_type` varchar(40) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL DEFAULT '' COMMENT '大分類鍵字（須符合 ItemTable）',
  `use_type` varchar(20) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL DEFAULT '' COMMENT '使用類型鍵字（須符合 ItemTable._useTypes）',
  `material` varchar(45) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL DEFAULT '' COMMENT '材質鍵字（須符合 ItemTable._materialTypes）',
  `weight` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '重量',
  `invgfx` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '背包圖形 ID',
  `grdgfx` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '地面圖形 ID',
  `itemdesc_id` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '說明編號',
  `stackable` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '0=不可堆疊 1=可堆疊',
  `max_charge_count` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '充能／次數上限；0=不用',
  `min_lvl` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '最低等級；0=不限',
  `max_lvl` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '最高等級；0=不限',
  `bless` int(10) UNSIGNED NOT NULL DEFAULT 1 COMMENT '0一般 1祝福 2詛咒 3幻象（內部值）',
  `trade` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '1=不可交易',
  `cant_delete` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '1=不可刪除',
  `delay_id` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '共用冷卻群組 ID',
  `delay_time` int(10) NOT NULL DEFAULT 0 COMMENT '共用冷卻秒數（毫秒）',
  `delay_effect` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '道具使用延遲（毫秒）',
  `food_volume` int(10) UNSIGNED NOT NULL DEFAULT 0 COMMENT '食物相關數值',
  `save_at_once` tinyint(3) UNSIGNED NOT NULL DEFAULT 1 COMMENT '1=狀態變更立即存檔',
  `limit_class` int(11) NOT NULL DEFAULT 127 COMMENT '1=王族；2=騎士；4=妖精；8=法師；16=黑妖；32=龍騎；64=幻術；127=全職業',
  PRIMARY KEY (`item_id`) USING BTREE,
  INDEX `item_id`(`item_id`) USING BTREE
) ENGINE = MyISAM AUTO_INCREMENT = 561536 CHARACTER SET = utf8 COLLATE = utf8_general_ci ROW_FORMAT = DYNAMIC;

INSERT INTO `etcitem` VALUES (NULL, 42501, 'GM瞬間移動', 'wand.Storm_Walk', 'GM瞬間移動', 'wand', 'spell_long', 'paper', 0, 828, 11, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 127);
