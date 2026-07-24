# Fenix
## Rules
### Todo
- [ ] General reconstruction (rule 8): after a General is captured, next turn may create one from two adjacent Soldiers
- [ ] Game over on king loss (rule 10): if King captured and cannot recreate, you lose
- [ ] Threefold repetition draw (rules 11-12): repetition detection + mutual king-capture-impossible draw
### Doing
- [ ] Mandatory captures (rule 6): capture generation works, needs enforcement in is_legal()
- [ ] Chain capture execution (rule 6): capture path generation done, needs game-loop execution
- [ ] Best capture rule (rule 7): algorithm exists in capture.rs, needs integration into game loop
- [ ] King reconstruction precedence (rule 9): forces_king_when_missing rule filter exists, needs game-engine enforcement
### Done
- [x] Board representation & FEN (rule 1-2): 9x9 board, starting position, FEN parse/generate
- [x] Piece representation (rule 1): bit-packed Square with color, kind, weight, upgrade path
- [x] Movement rules (rule 5): Soldier (1 orthogonal), General (rook-like sliding), King (1 any-direction)
- [x] Capture generation (rule 6): jump logic for all piece types, chain capture recursion
- [x] Capture scoring (rule 7): weight-based optimal path selection (King=3, General=2, Soldier=1)
- [x] Setup phase restriction (rule 4): only Upgrade actions allowed during first 5 turns
- [x] Material limits (rule 4): max 3 Generals, max 1 King enforced in is_legal()
- [x] Forced king creation (rule 9): is_legal() filters to Upgrade-only when King is missing
