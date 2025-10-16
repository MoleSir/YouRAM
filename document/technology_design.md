# 工艺库设计

## 电路类型

电路分为三种类型：

- LefCell：接口由 YouRAM 规定，用户必须提供满足接口的电路文件
- StdCell：一些数字单元，INV、AND、OR 等，这些实现由 pdk 自己实现，需要提供：
    -  .lib 文件：读取每个标准单元电路的功能
    - .sp 文件：标准单元的电气连接
    - .gds/.lef 文件：版图实现（用来布局布线）
- Design：YouRAM 自己利用 LefCell 或 StdCell 生成的模块电路，具有层次结构



## 程序接口设计

### LefCell

Bitcell（内存单元）、SenseAmp（敏感放大器）、Precharge（预充电）、WriteDriver（写驱动） 以及 TriGate（三态门）

读取 .sp：检查用户提供的端口是否与 YouRAM 要求一致。同时保存解析得到的 spice 文件结构（其中不应该出现子电路实例）。

读取 .gds：有些单元的宽度需要进行比较，同时读取版图信息

```rust
pub struct BitCell {
	name: String,
    bl_port: Port,
    br_port: Port,
    wl_port: Port,
    vdd_port: Port,
    gnd_port: Port,
}
```

由于这些 LefCell 都不需要参数，并且与工艺库强相关，并且工艺库加载阶段就会全部读入，所以可以创建的接口就设计到 Pdk 对象中：

````rust
impl Pdk {
	pub fn get_bitcell(&self) -> Rc<BitCell>;
	....
}
````

### StdCell

StdCell 需要有不同类型：Inv（反相器）、AND（与门）、OR（或门）、NAND（与非门）、NOR（或非门）、DFF（D 触发器）

每个类型都有不同的驱动能力、有些还有输入 pin 的数量差别。

首先读取 .lib 文件，获取不同电路属于哪些 StdCell。比如遍历所有 cell，所有 function 为 & 的就是 AND，然后在这些里面区分 pin 的数量，都确定后再确定驱动能力（名称区别、area 区别？）。

麻烦的是，我们对这些电路一无所知，pin 的名称、顺序都需要读取 pdk 后才可以。

所以连接电路的时候，对 StdCell，我们无法保证其 spice 的连接顺序，不可以直接在代码中写

```rust
pub struct Inv {
	name: String,
	drive_strength: DriveStrength,
    
   	input_port: Port,
    output_port: Port,
    vdd_port: Port,
    gnd_port: Port,
   	
   	netlist: Netlist,
    layout: Layout,
}

pub struct And {
    name: String,
	drive_strength: DriveStrength,
	input_ports: Vec<Port>,
    output_port: Port,
    vdd_port: Port,
    gnd_port: Port,
    
   	netlist: Netlist,
    layout: Layout,
}

pub enum DriveStrength {
	X1,
	X2,
	X4,
	...
}
```

也是需要向 pdk 进行申请：

````rust
impl Pdk {
	pub fn get_inv(drive_strength: DriveStrength) -> Rc<Inv>;
    pub fn get_and(input_size: usize, drive_strength: DriveStrength) -> Rc<And>;
    ...
}
````

### Module

````
````

定义一个 Module trait， 每个电路是单独的一个类型实例

因为我们分类比较多，如何管理电路是比较棘手的

````rust
pub enum CircuitRc {
	Module(Rc<dyn Module>),
	Library(Rc<dyn Library>),
}


````

但对 Instance 来说，其是一个电路的实例，不管是 Module 还是 Library，



````rust
pub trait Circuit {
	fn ports(&self) -> &[Rc<Port>];
    fn get_port(&self, name: &str) -> Option<Rc<Port>>;
}

pub trait Module : Circuit {
    fn instance(&self) -> &[Rc<Instance>];
 	fn sub_circuit(&self) -> &[Rc<Circuit>];
    fn 
}

pub trait Library: Circuit {
	fn 
}
````







> 接口：电路端口的名称、数量。在 spice 中定义的端口顺序，在版图文件中的大致规划