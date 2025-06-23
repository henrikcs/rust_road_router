## Input Converted to RoutingKit format

The files

- `first_ipp_of_arc`
- `first_out`
- `head`
- `ipp_departure_time`
- `ipp_travel_time`

have been generated with the following command (executed in the root directory of the project):

`/target/debug/import_tpgr ./import/tpgr/example_data/small_road_network.tpgr ./data/imported/tpgr/example_data/small_road_network`

The edges in the input should be sorted by the tail's node id (i.e. by node index)

## Additional Input consisting of Latitude/Longitude Data

The files

- `latitude`
- `longitude`

have been generated with the following command (executed in the root directory of the project):

`./target/debug/import_lat_lon ./import/tpgr/example_data/small_road_network.latlon ./data/imported/tpgr/example_data/small_road_network/`

## Output of InertialFlowCutter

The file `cch_perm` has been generated with the following command (executed in the root directory of the project):

`./flow_cutter_cch_order.sh ./data/imported/tpgr/example_data/small_road_network $(nproc --all)`

## Output Metric Independent Preprocessing

The files

- `cch/cch_first_out`
- `cch/cch_head`
- `cch/cch_ranks`

have been generated with the following command (executed in the root directory of the project):

`./target/debug/tdcch_static_preprocessing ./data/imported/tpgr/example_data/small_road_network`

## Output of Customization

The files in the folder `customized` have been generated with the following command (executed in the root directory of the project):

`./target/debug/tdcch_customization ./data/imported/tpgr/example_data/small_road_network`
