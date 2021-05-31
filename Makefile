all:
	@cp ./os.bin ./k210.bin

alla:
	@cd ./os && cargo vendor --offline
	@cd ./os && make build BOARD=k210
	@cp ./os/target/riscv64gc-unknown-none-elf/release/os.bin ./k210.bin

run:
	@cd ./os && make run BOARD=k210