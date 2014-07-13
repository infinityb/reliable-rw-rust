
all: reliable-write reliable-encap



reliable-write: reliable-rw
	cp -f $< $@


reliable-encap: reliable-rw
	cp -f $< $@


reliable-rw: reliable_rw.rs
	rustc -O -o reliable-rw reliable_rw.rs


clean:
	rm reliable-rw reliable-encap reliable-write


test:
	rustc --test -o reliable_rw_test reliable_rw.rs
	./reliable_rw_test
