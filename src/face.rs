use crate::{delay, App};

/// 表情
#[derive(Debug)]
pub struct Face {
    pub ram: [u8; 8],
}

impl Face {
    pub fn new() -> Self {
        Self {
            ram: [
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                0b00000000,
            ],
        }
    }

    //在初级显存里写入指定坐标信息，数组最左下面为原点
    pub fn work(&mut self, x: u8, y: u8, state: u8) {
        //左下角为原点
        //坐标系方向变换
        //充电口为下（默认坐标系是充电口朝右，所以需要把坐标系逆时针旋转90度）
        let x_m: u8 = 1; //用来做移位运算缓存
        let x_b: u8 = self.ram[7 - y as usize];
        //检查参数合理性
        if x > 7 || y > 7 {
            return;
        }

        if state == 1 {
            //点亮
            self.ram[7 - y as usize] = x_b | (x_m << (7 - x));
        } else if state == 0 {
            //熄灭
            self.ram[7 - y as usize] = x_b & (!(x_m << (7 - x)));
        }
    }

    pub fn clear(&mut self) {
        self.ram.iter_mut().for_each(|r| *r = 0);
    }

    /// 呆滞眼
    pub fn slack_eyes(&mut self, x: u8, y: u8) {
        // 左眼
        self.work(x, y, 1);
        self.work(x, y + 1, 1);
        self.work(x + 1, y, 1);
        self.work(x + 1, y + 1, 1);

        // 右眼
        self.work(x + 4, y, 1);
        self.work(x + 4, y + 1, 1);
        self.work(x + 5, y, 1);
        self.work(x + 5, y + 1, 1);
    }

    /// 闭眼
    pub fn close_eyes(&mut self) {
        // 左眼
        self.work(0, 4, 1);
        self.work(1, 4, 1);
        self.work(2, 4, 1);

        // 右眼
        self.work(5, 4, 1);
        self.work(6, 4, 1);
        self.work(7, 4, 1);
    }

    /// 大笑眼
    pub fn laugh_eyes(&mut self) {
        // 左眼
        self.work(0, 4, 1);
        self.work(1, 5, 1);
        self.work(2, 4, 1);

        // 右眼
        self.work(5, 4, 1);
        self.work(6, 5, 1);
        self.work(7, 4, 1);
    }

    /// 生气眼
    pub fn angry_eyes(&mut self) {
        // 左眼
        self.work(1, 4, 1);
        self.work(1, 6, 1);
        self.work(2, 5, 1);
        self.work(3, 4, 1);

        // 右眼
        self.work(4, 4, 1);
        self.work(5, 5, 1);
        self.work(6, 6, 1);
        self.work(6, 4, 1);
    }

    /// 虚眼
    pub fn slightly_closed_eyes(&mut self) {
        // 左眼
        self.work(1, 3, 1);
        self.work(1, 4, 1);
        self.work(2, 4, 1);
        self.work(0, 4, 1);

        // 右眼
        self.work(6, 3, 1);
        self.work(5, 4, 1);
        self.work(6, 4, 1);
        self.work(7, 4, 1);
    }

    /// 呆滞嘴
    pub fn slack_mouth(&mut self) {
        self.work(3, 2, 1);
        self.work(4, 2, 1);
    }

    /// 无奈嘴
    pub fn powerless_mouth(&mut self) {
        self.work(2, 1, 1);
        self.work(3, 1, 1);
        self.work(4, 1, 1);
        self.work(5, 1, 1);
    }

    /// 嘟嘴
    pub fn pout_mouth(&mut self) {
        self.work(3, 1, 1);
        self.work(3, 2, 1);
        self.work(4, 1, 1);
        self.work(4, 2, 1);
    }

    /// 惊恐嘴
    pub fn terrify_mouth(&mut self) {
        self.work(2, 1, 1);
        self.work(2, 2, 1);
        self.work(3, 0, 1);
        self.work(3, 3, 1);
        self.work(4, 0, 1);
        self.work(4, 3, 1);
        self.work(5, 1, 1);
        self.work(5, 2, 1);
    }

    /// 大笑嘴
    pub fn laugh_mouth(&mut self) {
        self.work(3, 1, 1);
        self.work(4, 1, 1);
        self.work(2, 2, 1);
        self.work(5, 2, 1);
    }

    /// 生气嘴
    pub fn angry_mouth(&mut self) {
        self.work(3, 2, 1);
        self.work(4, 2, 1);
        self.work(2, 1, 1);
        self.work(5, 1, 1);
    }

    /// 休眠表情
    pub fn dormancy_face(&mut self, app: &App) {
        // byte beeper_old = beeper;
        // beeper = 1;
        let ex: u8 = 1;
        let ey: u8 = 4;

        //while(1){//动作循环开始
        ///////////东张西望
        //呆滞嘴
        self.clear();
        self.slack_mouth();
        //呆滞眼
        self.slack_eyes(ex, ey);
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 500; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        // if (beeper) {
        //   tone(10, 6000, 50);
        // }
        //呆滞眼左看
        self.clear();
        self.slack_eyes(ex - 1, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 400; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        //呆滞眼右看
        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 10; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //

        self.clear();

        // if (beeper) {
        //   tone(10, 6000, 50);
        // }
        self.slack_eyes(ex + 1, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 500; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //

        //眼神复位
        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 500; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        /////////////眨眼后微笑再眨眼
        //微笑
        self.clear();
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 1000; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        //眨眼
        self.clear();
        self.close_eyes();
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        // //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 80; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        // if (beeper) {
        //   tone(10, 6000, 50);
        // }
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 500; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //眨眼
        self.clear();
        self.close_eyes();
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 80; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        // if (beeper) {
        //   tone(10, 6000, 50);
        // }
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 500; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //眨眼

        self.clear();
        self.close_eyes();
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 80; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        // if (beeper) {
        //   tone(10, 6000, 50);
        // }
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 900; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //

        /////////////平静地说话
        //呆滞嘴
        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //嘟嘴

        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //嘟嘴

        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //嘟嘴

        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //说第二句

        //呆滞嘴
        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //嘟嘴

        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //嘟嘴

        self.clear();
        // if (beeper) {
        //   tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //嘟嘴

        self.clear();
        // if (beeper) {
        //     tone(10, random(3000, 9000), 50);
        // }
        self.slack_eyes(ex, ey);
        self.pout_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 200; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //呆滞嘴

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 800; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        ///////////////眨眼等待
        //眨眼
        self.clear();
        // if (beeper) {
        //   tone(10, 8000, 50);
        // }
        self.close_eyes();
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 700; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        //眨眼

        self.clear();
        // if (beeper) {
        //   tone(10, 5000, 50);
        // }
        self.close_eyes();
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 1000; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        // beeper = beeper_old;
        //重复整个过程
        //}//循环结束
    }

    /// 唤醒表情
    pub fn rouse_face(&mut self) {
        // byte beeper_old = beeper;
        // beeper = 1;
        let ex: u8 = 1;
        let ey: u8 = 4;
        self.clear();
        ///////////////眨眼等待
        //眨眼
        // if (beeper) {
        //   tone(10, 8000, 50);
        // }
        self.close_eyes();
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 700; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        //眨眼
        self.clear();
        // if (beeper) {
        //   tone(10, 5000, 50);
        // }
        self.close_eyes();
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();

        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 100; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }
        self.clear();
        self.slack_eyes(ex, ey);
        self.slack_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        //设置每次动画结束后的间隔时长
        // for (int i = 0; i < 1000; i++) {
        //   zhonglifangxiang_panduan();
        //   if (zhonglifangxiang_old != lc.zhonglifangxiang) {  //当重力方向改变时刷新画面
        //     lc.bitmap(lc.LedBuffer_work);                     //  让充电池上方对着充电口
        //     lc.UpLoad();
        //   }
        // }

        // beeper = beeper_old;
    }

    /// FIXME 破记录
    pub fn break_record_face(&mut self, app: &App) {
        // TODO 开启声音
        let beeper: u8 = 1;

        let ex = 1;
        let ey = 4;

        //惊讶嘴
        self.clear();
        self.terrify_mouth();
        //呆滞眼
        self.slack_eyes(ex, ey);
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(500);

        //眨眼
        self.clear();
        // if (beeper) {
        //     tone(10, 8000, 50);
        // }
        self.close_eyes();
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(100);

        self.clear();
        self.slack_eyes(ex, ey);
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(700);

        //眨眼
        self.clear();
        // if (beeper) {
        //     tone(10, 8000, 50);
        // }
        self.close_eyes();
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(100);

        self.clear();
        self.slack_eyes(ex, ey);
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(700);

        //眨眼
        self.clear();
        // if (beeper) {
        //     tone(10, 8000, 50);
        // }
        self.close_eyes();
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(100);

        self.clear();
        self.slack_eyes(ex, ey);
        self.terrify_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(700);

        //微笑
        self.clear();
        self.slack_eyes(ex, ey);
        self.laugh_mouth();
        // zhonglifangxiang_panduan();
        // lc.bitmap(lc.LedBuffer_work);
        // lc.UpLoad();
        delay(1000);
        // beeper = beeper_old;
    }
}
