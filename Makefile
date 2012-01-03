SRCDIR=../rust/src/llvm/tools/clang/include
LIBDIR=../rust/build-make/llvm/x86_64-apple-darwin/Release+Asserts/lib

CFLAGS=-I${SRCDIR}
LDFLAGS=-L${LIBDIR} -lclang

all: librustclang.a

rustclang.o: rustclang.c
	gcc -I${SRCDIR} -c -fPIC -o $@ $<

librustclang.a: rustclang.o
	ar -r $@ $<

ctest: libprint.a
	gcc -g ${CFLAGS} ${LDFLAGS} test.c
	DYLD_LIBRARY_PATH=${LIBDIR} ./a.out

clean:
	rm -rf *.o *.a
