all:
	@cd ./os && make build BOARD=k210
	@cp ./os/target/riscv64gc-unknown-none-elf/release/os ./k210.bin

run:
	@cd ./os && make run BOARD=k210