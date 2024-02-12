DATA_DIR := tests/data
OSM_DIR := $(DATA_DIR)/osm
GEOJSON_DIR := $(DATA_DIR)/test_geojson
FMI_DIR := $(DATA_DIR)/fmi

INTERNET_OSM := https://cloud.p-fruck.de/s/pf9JfNabwDjrNL8/download/planet-coastlinespbf-cleaned.osm.pbf
NETWORK_OSM := $(OSM_DIR)/planet-coastlines.osm.pbf
PLANET := $(GEOJSON_DIR)/planet.geojson
NETWORK_GEOJSON := $(GEOJSON_DIR)/network.geojson

NETWORK_FMI := $(FMI_DIR)/network.gr

dirs:
	mkdir tests/data/test_geojson/
	mkdir tests/data/image/
	mkdir tests/data/osm/
	mkdir tests/data/fmi/


download:
	curl $(INTERNET_OSM) -o $(NETWORK_OSM)

convert_osm:
	cargo run --release --bin convert_osm --\
		--input $(NETWORK_OSM)\
		--output $(PLANET)

generate_network:
	cargo run --release --bin generate_network --\
		--input $(PLANET)\
		--num-nodes 4000000\
		--output-network $(NETWORK_FMI)\
		--output-geojson $(NETWORK_GEOJSON)\
		--output-image tests/data/test_geojson/network.png

