
all: reliable-write reliable-encap


reliable-write: reliable_write.rs reliable_rw_common.rs sha256.rs
	rustc -O -o reliable-write reliable_write.rs


reliable-encap: reliable_encap.rs reliable_rw_common.rs sha256.rs
	rustc -O -o reliable-encap reliable_encap.rs


clean:
	rm -f reliable-encap reliable-write


reliable-encap_test: reliable_encap.rs reliable_rw_common.rs sha256.rs
	rustc --test -o reliable-encap_test reliable_encap.rs


reliable-rw_test: reliable_write.rs reliable_rw_common.rs sha256.rs
	rustc --test -o reliable-rw_test reliable_write.rs


test: reliable-encap_test reliable-rw_test
	./reliable-encap_test
	./reliable-rw_test

