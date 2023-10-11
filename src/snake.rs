/// 贪吃蛇
#[derive(Debug)]
pub struct Snake {
    /// 头和尾的坐标

    /// 用来标记当前蛇的运动方向,上1，右2，下3，左4
    direction: u8,
    /// 用来存储蛇身体坐标信息，定义：* * ｜0 0 0｜0 0 0
    ///                             不 用 |x坐 标|y坐标
    body: [u8; 64],
    /// 保存最新头在body数组里的位置
    head_addr: u8,
    /// 保存最新尾在body数组里的位置
    tail_addr: u8,
    /// 记录游戏得分
    pub score: u8,
    /// 食物的坐标，定义同上
    pub food_xy: u8,
    /// 取值范围0-7，外部绘图使用
    pub head_x: u8,
    /// 取值范围0-7，外部绘图使用
    pub head_y: u8,
    /// 取值范围0-7,外部绘图使用
    pub tail_x: u8,
    /// 取值范围0-7，外部绘图使用
    pub tail_y: u8,
    /// 标记蛇身是否增长
    pub body_added: u8,
    /// 标记是否吃到食物
    pub food_flag: u8,
    /// 定义蛇身体长度
    pub body_length: u8,
    /// 标记蛇的生命状态，生存1还是死亡0
    pub life_state: u8,
    /// 整合头坐标为字节
    pub head_xy: u8,
    pub tail_xy: u8,
    pub tail_xy_del: u8,
}

impl Snake {
    fn refresh_body() {
        // switch(food_flag){
        //
        // 	case 0://当什么都没发生的时候
        //
        // 		//保存蛇尾坐标信息
        // 		wei_xy=she_body[wei_adr];
        //
        // 		//更新蛇尾位置
        // 		if(wei_adr<she_body_length-1){
        // 			wei_adr++;//如果没触边就变为后面一个位置
        //
        // 		}else if(wei_adr==she_body_length-1){
        // 			wei_adr=0;//如果触边，就归零
        // 		}
        //
        // 		//更新蛇头位置
        // 		if(tou_adr<she_body_length-1){
        // 			tou_adr++;//如果没触边就变为后面一个位置
        //
        // 		}else if(tou_adr==she_body_length-1){
        // 			tou_adr=0;//如果触边，就归零
        // 		}
        //
        // 		//在新蛇头位置写入当前蛇头坐标信息
        // 		she_body[tou_adr]=tou_xy;
        //
        // 		//头尾坐标解算，用于绘制
        // 		tou_x=tou_xy>>3;
        // 		tou_y=tou_xy&0B00000111;
        // 		wei_x=wei_xy>>3;
        // 		wei_y=wei_xy&0B00000111;
        // 		body_add=0;
        //
        // 		break;
        //
        // 	case 1://当吃到食物的时候ok
        //
        //
        // 		she_body_reset();//先整理蛇身数组
        //
        // 		tou_adr=she_body_length;//更新蛇头位置
        //
        // 		she_body[tou_adr]=tou_xy; //更新蛇头坐标
        //
        // 		wei_adr=0;//重置蛇尾位置尾0
        // 		wei_xy=she_body[wei_adr];
        // 		she_body_length++;//蛇身长度加1
        // 		//头尾坐标解算，用于绘制
        // 		tou_x=tou_xy>>3;
        // 		tou_y=tou_xy&0B00000111;
        // 		wei_x=wei_xy>>3;
        // 		wei_y=wei_xy&0B00000111;
        //
        // 		food_create();//重新生成食物
        // 		body_add=1;
        //
        // 		break;
        //
        //
        // }
    }

    /// 检测是否超出边界，撞到自己，吃到食物，蛇头每更新一次就运行一次
    fn collision_detection() {
        //检测当前头坐标是否超出边界，是否撞到自己，是否吃到食物,被move函数调用
        //
        //
        //
        // //如果当前新坐标超出屏幕边缘，死路一条
        // if(tou_x>7||tou_x<0||tou_y>7||tou_y<0){
        //
        // 	she_life_state=0;//
        // }
        //
        // //判断蛇是不是撞上了自己
        // if(she_life_state==1){
        // 	for(int i=0;i<she_body_length;i++){
        //
        // 		if(tou_xy==she_body[i]){//如果撞上自己，死路一条
        // 				she_life_state=0;
        // 				i=she_body_length;
        //
        // 		}
        // 	}
        // }
        // //没撞上自己？那看是不是撞上食物
        // if(she_life_state==1){
        // 	if(tou_xy==food_xy){//如果坐标和食物重合
        //
        // 		if(she_body_length<63){
        //
        // 			food_flag=1;//吃到食物标志为1
        //
        // 			defen++;
        // 			//delay(1000);
        // 		}
        //
        // 	}
        // }
        //
        // if(she_life_state==1){
        // 	//更新一次body数组
        // 	refresh_body();//根据
        //
        // }
    }

    /// 重新整理蛇身体数组的顺序
    fn reset_body() {
        //        byte she_body_temp[64];//一个临时数组用来重新整理蛇身数据的顺序，整理完了再按顺序写回原数组
        //
        // for(int i=wei_adr;i<she_body_length;i++){
        // 	she_body_temp[i-wei_adr]=she_body[i];//把蛇尾坐标到蛇身长度所及位置的数据存从零存入，
        //
        // }
        // for(int i=0;i<=tou_adr;i++){
        // 	she_body_temp[i+she_body_length-wei_adr]=she_body[i];//把蛇头左边的数据也整理进去
        //
        // }
        //
        // for(int i=0;i<she_body_length;i++){//把数据再复制回原数组
        //
        // she_body[i]=she_body_temp[i];
        //
        //
        // }
    }

    /// 创建食物坐标，不能与当前蛇身重叠
    fn food_create() {
        //更新食物坐标
        // 	byte food;
        // 	byte flag=1;
        // while(flag){
        // 	food=0;
        // 	//randomSeed(analogRead(A0));//根据模拟口更新随机数种子
        // 	food=food|(random(0,7)<<3);//生成x坐标
        // 	food|=random(0,7);//生成y坐标
        // 	flag=0;
        // 	for(int i=0;i<she_body_length;i++){
        // 		if(food==she_body[i]){
        // 			flag=1;//如果坐标和身体重合，就立即退出循环重新开始生成
        //
        // 			break;
        // 		}
        //
        //
        // 	}
        // }
        //
        // food_xy=food;
        //
        //
        // food_flag=0;//食物标志归零
    }

    pub fn new() -> Self {
        Self::food_create();

        let mut snake = Self {
            direction: 1,
            body: [0; 64],
            head_addr: 0,
            tail_addr: 0,
            score: 0,
            food_xy: 0,
            head_x: 3,
            head_y: 3,
            tail_x: 0,
            tail_y: 0,
            body_added: 0,
            food_flag: 0,
            body_length: 1,
            life_state: 1,
            head_xy: 0,
            tail_xy: 0,
            tail_xy_del: 0,
        };

        snake.head_xy = (snake.head_x << 3) | snake.head_y;
        snake.body[snake.head_addr as usize] = snake.head_xy;
        snake.tail_xy = snake.head_xy;
        snake.tail_x = snake.tail_xy >> 3;
        snake.tail_y = snake.tail_xy & 0b00000111;
        snake
    }

    /// 根据按键标志位修改蛇的运动方向标识位
    /// 根据按键标志位修改蛇的运动方向fangxiang，
    /// 外部标识位key_scan.key_flag,上1，右2，下3，左4
    pub fn key_direction(&mut self, key_flag: u8) {
        // switch(fangxiang){//根据当前方向和按键输入，更新蛇头移动方向
        // 	case 1:
        // 		switch(key_flag){
        // 			case 2:
        // 				fangxiang=2;
        // 				break;
        // 			case 4:
        // 				fangxiang=4;
        // 				break;
        // 		}
        // 		break;
        // 	case 2:
        // 		switch(key_flag){
        // 			case 1:
        // 				fangxiang=1;
        // 				break;
        //
        // 			case 3:
        // 				fangxiang=3;
        // 				break;
        // 		}
        // 		break;
        // 	case 3:
        // 		switch(key_flag){
        // 			case 2:
        // 				fangxiang=2;
        // 				break;
        // 			case 4:
        // 				fangxiang=4;
        // 				break;
        // 		}
        // 		break;
        // 	case 4:
        // 		switch(key_flag){
        // 			case 1:
        // 				fangxiang=1;
        // 				break;
        // 			case 3:
        // 				fangxiang=3;
        // 				break;
        // 		}
        // 		break;
        // }
    }

    /// 根据fangxiang标志位移动一次
    /// 得到一个新的蛇头坐标
    pub fn r#move() {
        //        switch(fangxiang){
        //
        // 	case 1://向上，则y递增
        //
        //
        //
        // 			tou_y++;
        // 			tou_xy=(tou_x<<3)|tou_y;
        //
        //
        // 		break;
        // 	case 2://向右，则x递增
        //
        // 			tou_x++;
        // 			tou_xy=(tou_x<<3)|tou_y;
        //
        //
        // 		break;
        // 	case 3://向下，则y递减
        //
        // 			tou_y--;
        // 			tou_xy=(tou_x<<3)|tou_y;
        //
        // 		break;
        //
        // 	case 4://向左，则x递减
        //
        // 			tou_x--;
        // 			tou_xy=(tou_x<<3)|tou_y;
        //
        // 		break;
        // }
        //
        // pengzhuang_jiance();//根据当前新的坐标，判断蛇的状态，要么死，要么吃到食物，要么什么都不发生
    }

    /// 重置
    pub fn reset(&mut self) {
        //初始化头和尾的坐标，刚开始长度为零，所以头和尾重叠
        // 清空身体信息
        for i in 0..64 {
            self.body[i] = 0;
        } // 保存最新头在body数组里的位置
        self.head_addr = 0;
        // 保存最新尾在body数组里的位置
        self.tail_addr = 0;
        self.head_x = 3;
        self.head_y = 3;
        self.head_xy = (self.head_x << 3) | self.head_y;
        self.body[self.head_addr as usize] = self.head_xy;
        self.tail_xy = self.head_xy;
        self.tail_x = self.tail_xy >> 3;
        self.tail_y = self.tail_xy & 0b00000111;
        self.score = 0;
        // 初始化移动方向上1，右2，下3，左4
        self.direction = 1;

        self.body_added = 0;
        // 定义蛇身体长度
        self.body_length = 1;
        self.life_state = 1;

        // 随机初始化食物坐标
        Self::food_create();
    }
}
