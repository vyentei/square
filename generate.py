#!/usr/bin/python3

import fontforge
import psMat as psmat
import math

TAU = math.tau

def generate(sfd, output):
    gen_flags = ("opentype", "old-kern")

    font = fontforge.open(sfd)
    font.generate(output, "", gen_flags)

    return

def main():
    generate("./Square.sfd", "./VyenteiSquare.ttf")
    generate("./Square.sfd", "./VyenteiSquare.woff")

    return

main()
