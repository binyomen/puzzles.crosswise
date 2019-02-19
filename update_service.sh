#!/bin/sh

# update the container images
docker pull nginx
docker pull bweedon/puzzles.crosswise

# stop the containers
docker stop nginx
docker stop puzzles

# remove the containers
docker rm nginx
docker rm puzzles

# remove the network
docker network rm puzzles_network

# recreate the network
docker network create --driver bridge puzzles_network

# start the containers back up
docker run --name nginx --net puzzles_network -p 80:80 -v ~/puzzles.crosswise/nginx.conf:/etc/nginx/conf.d/nginx.conf --restart always -d nginx
docker run --name puzzles --net puzzles_network --restart always -d bweedon/puzzles.crosswise
