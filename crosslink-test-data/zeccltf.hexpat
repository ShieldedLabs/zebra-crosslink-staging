struct TFHdr {
    char magic[8];
    u64 instrs_o;
    u32 instrs_n;
    u32 instr_size;
};

struct TFSlice {
    u64 o;
    u64 size;
};

struct TFInstr {
    u32 kind;
    u32 flags;
    TFSlice data;
    u64 val[2];
};

struct TF {
    TFHdr hdr;
    u8 data[hdr.instrs_o - sizeof(hdr)];
    TFInstr instrs[hdr.instrs_n];
};
   
   
TF tf @0;