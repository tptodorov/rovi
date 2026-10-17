FROM ros:jazzy-ros-base
RUN apt-get update && apt-get install -y --no-install-recommends ros-jazzy-rmw-zenoh-cpp ros-jazzy-geometry-msgs && rm -rf /var/lib/apt/lists/*
ENV RMW_IMPLEMENTATION=rmw_zenoh_cpp
RUN apt-get update && apt-get install -y --no-install-recommends python3-pip && pip3 install --break-system-packages eclipse-zenoh && rm -rf /var/lib/apt/lists/*
