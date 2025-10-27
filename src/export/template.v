) (
    input                     clk,
    input                     csb,
    input                     we,
    input [ADDR_WIDTH-1 : 0]  addr,
    input [DATA_WIDTH-1 : 0]  din,
    output [DATA_WIDTH-1 : 0] dout
);

    // ------------------------ Memory ------------------------ //
    reg [DATA_WIDTH-1 : 0] memory [2**(ADDR_WIDTH)-1 : 0];

    // ------------------------ Register ------------------------ //
    reg                    csb_reg;
    reg                    we_reg;
    reg [ADDR_WIDTH-1 : 0] addr_reg;
    reg [DATA_WIDTH-1 : 0] din_reg;
    reg [DATA_WIDTH-1 : 0] dout_reg;

    always @(posedge clk) begin
        csb_reg   <= csb;
        we_reg    <= we;
        addr_reg  <= addr;
        din_reg <= din;
    end

    // ------------------------ Operation ----------------------- //
    always @(negedge clk) begin : read_operation
        if (csb_reg == 1'b0 && we_reg == 1'b0) begin
            dout_reg = memory[addr_reg];
        end
    end
    assign dout = dout_reg;
    
    always @(negedge clk) begin : write_operation
        if (csb_reg == 1'b0 && we_reg == 1'b1) begin
            memory[addr_reg] <= din_reg;
        end
    end

endmodule