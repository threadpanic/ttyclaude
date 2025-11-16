all :
	cargo build

clean :
	cargo clean

config :
	cp -v config.ini $$XDG_CONFIG_HOME/ttyclaude/config.ini

run :
	cargo run

.PHONY : clean all run config



