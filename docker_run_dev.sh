#!/usr/bin/env sh

docker volume create rpi_linux_gpio_driver_dev

docker run -it -v "./container_mount:/home/host_mount" -v "rpi_linux_gpio_driver_dev:/home/volume" --rm --name rpi_linux_gpio_driver_dev rpi_linux_gpio_driver_dev bash
