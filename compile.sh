#!/bin/bash

glslc shaders/shader.frag  -o shaders/frag.spv
glslc shaders/shader.vert  -o shaders/vert.spv
