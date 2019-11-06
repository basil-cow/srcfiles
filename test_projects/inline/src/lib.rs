mod a {
    #[path = "c"]
    mod b {
        mod d;
        mod e {
            #[path = "e/e.rs"]
            mod e;
        }
    }
}
