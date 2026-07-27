cmake_minimum_required(VERSION 3.20)
project(sample LANGUAGES CXX)

find_package(Threads REQUIRED)
find_package(OpenSSL REQUIRED)
include(GNUInstallDirs)

function(setup_target target)
  target_link_libraries(${target} PRIVATE Threads::Threads OpenSSL::SSL)
  if(BUILD_SHARED_LIBS)
    set_target_properties(${target} PROPERTIES POSITION_INDEPENDENT_CODE ON)
  endif()
endfunction()

function(add_component name)
  add_library(${name} STATIC ${ARGN})
  setup_target(${name})
endfunction()

macro(install_component name)
  install(TARGETS ${name}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
endmacro()

add_component(core src/core.cpp)
install_component(core)
