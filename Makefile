EXE = jackal
VER = X.X.X

ifeq ($(OS),Windows_NT)
	DATAGEN := datagen.exe
	TRAINER := trainer.exe
	DEV_NAME := $(EXE)-dev.exe
	X86_64_V2 := releases/$(EXE)-$(VER)-x86-64-v2.exe
	X86_64_V3 := releases/$(EXE)-$(VER)-x86-64-v3.exe
	X86_64_V4 := releases/$(EXE)-$(VER)-x86-64-v4.exe
else
	DATAGEN := datagen
	TRAINER := trainer
	DEV_NAME := $(EXE)-dev
	X86_64_V2 := releases/$(EXE)-$(VER)-x86-64-v2
	X86_64_V3 := releases/$(EXE)-$(VER)-x86-64-v3
	X86_64_V4 := releases/$(EXE)-$(VER)-x86-64-v4
endif

rule:
	cargo rustc --release --bin jackal -- -C target-cpu=native --emit link=$(DEV_NAME)

release:
	cargo rustc --release --bin jackal -- -C target-cpu=x86-64-v2 --emit link=$(X86_64_V2)
	cargo rustc --release --bin jackal -- -C target-cpu=x86-64-v3 --emit link=$(X86_64_V3)
	cargo rustc --release --bin jackal -- -C target-cpu=x86-64-v4 --emit link=$(X86_64_V4)

gen:
	cargo rustc --release --package datagen --bin datagen -- -C target-cpu=native --emit link=$(DATAGEN)

trainer:
	cargo rustc --release --package train --bin train -- -C target-cpu=native --emit link=$(TRAINER)