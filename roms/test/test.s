; L0001:	rts
; ldx     #255
; lda     #$00
; jmp     L0001

; LDA #$01
; STA $f0
; LDA #$cc
; STA $f1
; JMP ($00f0) ; dereferences to $cc01


LDA #$c0  ;Load the hex value $c0 into the A register
TAX       ;Transfer the value in the A register to X
INX       ;Increment the value in the X register
ADC #$c4  ;Add the hex value $c4 to the A register
BRK       ;Break - we're done