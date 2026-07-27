cmake_minimum_required(VERSION 3.20)
project(MyProject VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Threads REQUIRED)
find_package(OpenSSL REQUIRED)

include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

function(add_component name sources)
    add_library(${name} STATIC ${sources})
    target_include_directories(${name} PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
    )
    target_link_libraries(${name} PRIVATE Threads::Threads)
endfunction()

macro(setup_target target)
    set_target_properties(${target} PROPERTIES
        CXX_STANDARD 17
        POSITION_INDEPENDENT_CODE ON
    )
    if(MSVC)
        target_compile_options(${target} PRIVATE /W4 /WX)
    else()
        target_compile_options(${target} PRIVATE -Wall -Wextra -Werror)
    endif()
endmacro()

function(install_component name)
    install(TARGETS ${name}
        EXPORT ${name}Targets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endfunction()

add_component(core "src/core.cpp;src/util.cpp")
setup_target(core)

add_executable(my_app src/main.cpp)
target_link_libraries(my_app PRIVATE core OpenSSL::SSL)
setup_target(my_app)

if(BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

install_component(core)

function(scan_sources)
    foreach(src IN LISTS ALL_SOURCES)
        if(src MATCHES "test")
            continue()
        endif()
        if(src MATCHES "generated")
            break()
        endif()
        message(STATUS ${src})
    endforeach()
    while(NOT DONE)
        if(NOT ALL_SOURCES)
            return()
        endif()
    endwhile()
endfunction()
