cmake_minimum_required(VERSION 3.16)
project(sample LANGUAGES CXX)

find_package(Threads REQUIRED)
find_package(OpenSSL REQUIRED)
include(GNUInstallDirs)

function(add_component name)
  add_library(${name} STATIC ${name}.cpp)
  target_link_libraries(${name} PRIVATE Threads::Threads)
endfunction()

macro(setup_target name)
  if(NOT TARGET ${name})
    message(WARNING "missing target ${name}")
  endif()
endmacro()

function(install_component name)
  foreach(item ${ARGN})
    message(STATUS "installing ${item}")
  endforeach()
endfunction()

add_component(core)
setup_target(core)
install_component(core)
