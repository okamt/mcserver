use std::sync::OnceLock;

use getset::Getters;
use serde::Deserialize;

static REGISTRIES: OnceLock<Registries<'static>> = OnceLock::new();

pub fn registries() -> &'static Registries<'static> {
    REGISTRIES.get_or_init(|| {
        serde_json::from_slice(include_bytes!("assets/registries.min.json")).unwrap()
    })
}

#[derive(Deserialize, Debug, Getters)]
#[get = "pub"]
pub struct Registries<'a> {
    #[serde(borrow)]
    activity: Vec<&'a str>,
    #[serde(borrow)]
    advancement: Vec<&'a str>,
    #[serde(borrow)]
    armor_material: Vec<&'a str>,
    #[serde(borrow)]
    atlas: Vec<&'a str>,
    #[serde(borrow)]
    attribute: Vec<&'a str>,
    #[serde(borrow)]
    banner_pattern: Vec<&'a str>,
    #[serde(borrow)]
    block: Vec<&'a str>,
    #[serde(borrow)]
    block_definition: Vec<&'a str>,
    #[serde(borrow)]
    block_entity_type: Vec<&'a str>,
    #[serde(borrow)]
    block_predicate_type: Vec<&'a str>,
    #[serde(borrow)]
    block_type: Vec<&'a str>,
    #[serde(borrow)]
    cat_variant: Vec<&'a str>,
    #[serde(borrow)]
    chat_type: Vec<&'a str>,
    #[serde(borrow)]
    chunk_status: Vec<&'a str>,
    #[serde(borrow)]
    command_argument_type: Vec<&'a str>,
    #[serde(borrow)]
    creative_mode_tab: Vec<&'a str>,
    #[serde(borrow)]
    custom_stat: Vec<&'a str>,
    #[serde(borrow)]
    damage_type: Vec<&'a str>,
    #[serde(borrow)]
    data_component_type: Vec<&'a str>,
    #[serde(borrow)]
    datapack: Vec<&'a str>,
    #[serde(borrow)]
    decorated_pot_pattern: Vec<&'a str>,
    #[serde(borrow)]
    dimension: Vec<&'a str>,
    #[serde(borrow)]
    dimension_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_effect_component_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_entity_effect_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_level_based_value_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_location_based_effect_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_provider: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    enchantment_value_effect_type: Vec<&'a str>,
    #[serde(borrow)]
    entity_sub_predicate_type: Vec<&'a str>,
    #[serde(borrow)]
    entity_type: Vec<&'a str>,
    #[serde(rename = "experiment/bundle/advancement")]
    #[serde(borrow)]
    experiment_bundle_advancement: Vec<&'a str>,
    #[serde(rename = "experiment/bundle/recipe")]
    #[serde(borrow)]
    experiment_bundle_recipe: Vec<&'a str>,
    #[serde(rename = "experiment/trade_rebalance/enchantment_provider")]
    #[serde(borrow)]
    experiment_trade_rebalance_enchantment_provider: Vec<&'a str>,
    #[serde(rename = "experiment/trade_rebalance/loot_table")]
    #[serde(borrow)]
    experiment_trade_rebalance_loot_table: Vec<&'a str>,
    #[serde(rename = "experiment/trade_rebalance/tag/enchantment")]
    #[serde(borrow)]
    experiment_trade_rebalance_tag_enchantment: Vec<&'a str>,
    #[serde(rename = "experiment/trade_rebalance/tag/worldgen/structure")]
    #[serde(borrow)]
    experiment_trade_rebalance_tag_worldgen_structure: Vec<&'a str>,
    #[serde(borrow)]
    float_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    fluid: Vec<&'a str>,
    #[serde(borrow)]
    font: Vec<&'a str>,
    #[serde(borrow)]
    frog_variant: Vec<&'a str>,
    #[serde(borrow)]
    game_event: Vec<&'a str>,
    #[serde(borrow)]
    height_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    instrument: Vec<&'a str>,
    #[serde(borrow)]
    int_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    item: Vec<&'a str>,
    #[serde(borrow)]
    item_sub_predicate_type: Vec<&'a str>,
    #[serde(borrow)]
    jukebox_song: Vec<&'a str>,
    #[serde(borrow)]
    loot_condition_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_function_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_nbt_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_number_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_pool_entry_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_score_provider_type: Vec<&'a str>,
    #[serde(borrow)]
    loot_table: Vec<&'a str>,
    #[serde(borrow)]
    map_decoration_type: Vec<&'a str>,
    #[serde(borrow)]
    memory_module_type: Vec<&'a str>,
    #[serde(borrow)]
    menu: Vec<&'a str>,
    #[serde(borrow)]
    mob_effect: Vec<&'a str>,
    #[serde(borrow)]
    model: Vec<&'a str>,
    #[serde(borrow)]
    number_format_type: Vec<&'a str>,
    #[serde(borrow)]
    painting_variant: Vec<&'a str>,
    #[serde(borrow)]
    particle_type: Vec<&'a str>,
    #[serde(borrow)]
    point_of_interest_type: Vec<&'a str>,
    #[serde(borrow)]
    pos_rule_test: Vec<&'a str>,
    #[serde(borrow)]
    position_source_type: Vec<&'a str>,
    #[serde(borrow)]
    potion: Vec<&'a str>,
    #[serde(borrow)]
    recipe: Vec<&'a str>,
    #[serde(borrow)]
    recipe_serializer: Vec<&'a str>,
    #[serde(borrow)]
    recipe_type: Vec<&'a str>,
    #[serde(borrow)]
    rule_block_entity_modifier: Vec<&'a str>,
    #[serde(borrow)]
    rule_test: Vec<&'a str>,
    #[serde(borrow)]
    schedule: Vec<&'a str>,
    #[serde(borrow)]
    sensor_type: Vec<&'a str>,
    #[serde(borrow)]
    sound_event: Vec<&'a str>,
    #[serde(borrow)]
    stat_type: Vec<&'a str>,
    #[serde(borrow)]
    structure: Vec<&'a str>,
    #[serde(rename = "tag/banner_pattern")]
    #[serde(borrow)]
    tag_banner_pattern: Vec<&'a str>,
    #[serde(rename = "tag/block")]
    #[serde(borrow)]
    tag_block: Vec<&'a str>,
    #[serde(rename = "tag/cat_variant")]
    #[serde(borrow)]
    tag_cat_variant: Vec<&'a str>,
    #[serde(rename = "tag/damage_type")]
    #[serde(borrow)]
    tag_damage_type: Vec<&'a str>,
    #[serde(rename = "tag/enchantment")]
    #[serde(borrow)]
    tag_enchantment: Vec<&'a str>,
    #[serde(rename = "tag/entity_type")]
    #[serde(borrow)]
    tag_entity_type: Vec<&'a str>,
    #[serde(rename = "tag/fluid")]
    #[serde(borrow)]
    tag_fluid: Vec<&'a str>,
    #[serde(rename = "tag/game_event")]
    #[serde(borrow)]
    tag_game_event: Vec<&'a str>,
    #[serde(rename = "tag/instrument")]
    #[serde(borrow)]
    tag_instrument: Vec<&'a str>,
    #[serde(rename = "tag/item")]
    #[serde(borrow)]
    tag_item: Vec<&'a str>,
    #[serde(rename = "tag/painting_variant")]
    #[serde(borrow)]
    tag_painting_variant: Vec<&'a str>,
    #[serde(rename = "tag/point_of_interest_type")]
    #[serde(borrow)]
    tag_point_of_interest_type: Vec<&'a str>,
    #[serde(rename = "tag/worldgen/biome")]
    #[serde(borrow)]
    tag_worldgen_biome: Vec<&'a str>,
    #[serde(rename = "tag/worldgen/flat_level_generator_preset")]
    #[serde(borrow)]
    tag_worldgen_flat_level_generator_preset: Vec<&'a str>,
    #[serde(rename = "tag/worldgen/structure")]
    #[serde(borrow)]
    tag_worldgen_structure: Vec<&'a str>,
    #[serde(rename = "tag/worldgen/world_preset")]
    #[serde(borrow)]
    tag_worldgen_world_preset: Vec<&'a str>,
    #[serde(borrow)]
    texture: Vec<&'a str>,
    #[serde(borrow)]
    trigger_type: Vec<&'a str>,
    #[serde(borrow)]
    trim_material: Vec<&'a str>,
    #[serde(borrow)]
    trim_pattern: Vec<&'a str>,
    #[serde(borrow)]
    villager_profession: Vec<&'a str>,
    #[serde(borrow)]
    villager_type: Vec<&'a str>,
    #[serde(borrow)]
    wolf_variant: Vec<&'a str>,
    #[serde(rename = "worldgen/biome")]
    #[serde(borrow)]
    worldgen_biome: Vec<&'a str>,
    #[serde(rename = "worldgen/biome_source")]
    #[serde(borrow)]
    worldgen_biome_source: Vec<&'a str>,
    #[serde(rename = "worldgen/block_state_provider_type")]
    #[serde(borrow)]
    worldgen_block_state_provider_type: Vec<&'a str>,
    #[serde(rename = "worldgen/carver")]
    #[serde(borrow)]
    worldgen_carver: Vec<&'a str>,
    #[serde(rename = "worldgen/chunk_generator")]
    #[serde(borrow)]
    worldgen_chunk_generator: Vec<&'a str>,
    #[serde(rename = "worldgen/configured_carver")]
    #[serde(borrow)]
    worldgen_configured_carver: Vec<&'a str>,
    #[serde(rename = "worldgen/configured_feature")]
    #[serde(borrow)]
    worldgen_configured_feature: Vec<&'a str>,
    #[serde(rename = "worldgen/density_function")]
    #[serde(borrow)]
    worldgen_density_function: Vec<&'a str>,
    #[serde(rename = "worldgen/density_function_type")]
    #[serde(borrow)]
    worldgen_density_function_type: Vec<&'a str>,
    #[serde(rename = "worldgen/feature")]
    #[serde(borrow)]
    worldgen_feature: Vec<&'a str>,
    #[serde(rename = "worldgen/feature_size_type")]
    #[serde(borrow)]
    worldgen_feature_size_type: Vec<&'a str>,
    #[serde(rename = "worldgen/flat_level_generator_preset")]
    #[serde(borrow)]
    worldgen_flat_level_generator_preset: Vec<&'a str>,
    #[serde(rename = "worldgen/foliage_placer_type")]
    #[serde(borrow)]
    worldgen_foliage_placer_type: Vec<&'a str>,
    #[serde(rename = "worldgen/material_condition")]
    #[serde(borrow)]
    worldgen_material_condition: Vec<&'a str>,
    #[serde(rename = "worldgen/material_rule")]
    #[serde(borrow)]
    worldgen_material_rule: Vec<&'a str>,
    #[serde(rename = "worldgen/multi_noise_biome_source_parameter_list")]
    #[serde(borrow)]
    worldgen_multi_noise_biome_source_parameter_list: Vec<&'a str>,
    #[serde(rename = "worldgen/noise")]
    #[serde(borrow)]
    worldgen_noise: Vec<&'a str>,
    #[serde(rename = "worldgen/noise_settings")]
    #[serde(borrow)]
    worldgen_noise_settings: Vec<&'a str>,
    #[serde(rename = "worldgen/placed_feature")]
    #[serde(borrow)]
    worldgen_placed_feature: Vec<&'a str>,
    #[serde(rename = "worldgen/placement_modifier_type")]
    #[serde(borrow)]
    worldgen_placement_modifier_type: Vec<&'a str>,
    #[serde(rename = "worldgen/pool_alias_binding")]
    #[serde(borrow)]
    worldgen_pool_alias_binding: Vec<&'a str>,
    #[serde(rename = "worldgen/processor_list")]
    #[serde(borrow)]
    worldgen_processor_list: Vec<&'a str>,
    #[serde(rename = "worldgen/root_placer_type")]
    #[serde(borrow)]
    worldgen_root_placer_type: Vec<&'a str>,
    #[serde(rename = "worldgen/structure")]
    #[serde(borrow)]
    worldgen_structure: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_piece")]
    #[serde(borrow)]
    worldgen_structure_piece: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_placement")]
    #[serde(borrow)]
    worldgen_structure_placement: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_pool_element")]
    #[serde(borrow)]
    worldgen_structure_pool_element: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_processor")]
    #[serde(borrow)]
    worldgen_structure_processor: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_set")]
    #[serde(borrow)]
    worldgen_structure_set: Vec<&'a str>,
    #[serde(rename = "worldgen/structure_type")]
    #[serde(borrow)]
    worldgen_structure_type: Vec<&'a str>,
    #[serde(rename = "worldgen/template_pool")]
    #[serde(borrow)]
    worldgen_template_pool: Vec<&'a str>,
    #[serde(rename = "worldgen/tree_decorator_type")]
    #[serde(borrow)]
    worldgen_tree_decorator_type: Vec<&'a str>,
    #[serde(rename = "worldgen/trunk_placer_type")]
    #[serde(borrow)]
    worldgen_trunk_placer_type: Vec<&'a str>,
    #[serde(rename = "worldgen/world_preset")]
    #[serde(borrow)]
    worldgen_world_preset: Vec<&'a str>,
}
