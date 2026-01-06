let rand_idx = (fd -e rs | lines | uniq | length | random int 1..$in)
let rand_file = (fd -e rs | lines | uniq | get $rand_idx)

nvim $"($rand_file)"
