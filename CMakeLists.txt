cmake_minimum_required(VERSION 3.24)
project(DoctorAI VERSION 0.1.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.6 REQUIRED COMPONENTS Widgets Network)

qt_add_executable(DoctorAI WIN32
    main.cpp
    mainwindow.h
    mainwindow.cpp
    api_client.h
    api_client.cpp
)
target_link_libraries(DoctorAI PRIVATE Qt6::Widgets Qt6::Network)
