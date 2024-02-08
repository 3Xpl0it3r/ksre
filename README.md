# ksre 简介/定位
&emsp;`ksre`一款`kubernets`的sre工具,旨在提供最简单的安装,提供最便捷的操作,尽可能解决更多的的问题;
- 在k8s管理方面,由于市面上成熟的功能完备的dashboard很多(但是大多数功能我们未必都能用到),所以我们只提供一些最常用的k8s操作;用最省事的方式提供最简单的操作;
- 在troubleshooting方面市面功能强大的客观性平台也有很多,但是80%问题都属于基础问题,牛刀杀鸡大材小用,我们定位于用于杀鸡的刀;
&emsp;初衷希望用10%的精力去解决80%的问题;

# 安装/运行
```bash
git clone https://github.com/3Xpl0it3r/ksre.git
cd ksre
cargo  run --package ksre-tui
```

# 演示
![dashboard](./img/demo.gif)
