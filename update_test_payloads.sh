#!/bin/sh
set -x

mkdir -p tst-data/search
curl -X GET "https://esi.evetech.net/latest/search/?categories=character&datasource=tranquility&language=en-us&search=Necrothitude&strict=true" -H  "accept: application/json" -H  "Accept-Language: en-us" > tst-data/search/single-character.json