# docker registry 相关
IMAGE_NAME       = substrate-node
VERSION          = 0.0.15
# DRY
PUBLIC_REGISTRY_NAME = $(IMAGE_NAME):${VERSION}

.PHONY: image
image: # 打包
	docker build --compress -t ${PUBLIC_REGISTRY_NAME} .
