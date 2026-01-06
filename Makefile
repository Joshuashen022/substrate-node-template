# docker registry
IMAGE_NAME       = substrate-node
VERSION          = 0.0.15
# DRY
PUBLIC_REGISTRY_NAME = $(IMAGE_NAME):${VERSION}

.PHONY: image genesis
image: # 
	docker build --compress -t ${PUBLIC_REGISTRY_NAME} .

genesis: # generate genesis.json
	docker-compose run --rm build-spec

alice: # run alice
	rm -rf build/tmp/alice
	docker compose up alice

bob: # run bob
	rm -rf build/tmp/bob
	docker compose up bob

charlie: # run charlie
	rm -rf build/tmp/charlie
	docker compose up charlie