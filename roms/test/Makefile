build:
	ca65 *.s
	ld65 *.o -C test.cfg -o test.bin

clean:
	rm -f *.o test.bin

re:
	make clean && make