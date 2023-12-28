// import { mat4 } from 'gl-matrix';
// import * as opentype from 'opentype.js';
// import { GLYPHY_INFINITY } from './glyphy/util';
// import { add_glyph_vertices, GlyphInfo } from './glyphy/vertex';
import { get_char_arc_debug } from './pkg/pi_sdf.js';
// import { delete_glyph, set_glyph } from './sdf/glyph';
export const GLYPHY_INFINITY = Infinity;
export const GLYPHY_EPSILON = 1e-4;
/**
 * + 填充规则：奇偶规则
 * + 外围：（填充）顺时针，红-绿-蓝
 * + 内围：（挖空）逆时针，红-绿-蓝
 */
export class DrawText {
    init_x;
    init_y;
    size_x;
    size_y;

    // 鼠标上次点击的位置（相对Canvas的坐标）
    mouse_x;
    mouse_y;

    last_arc_count;
    last_bezier_count;

    last_blob_string;

    ctx;
    ttf;
    char;
    char_size;
    font;

    last_arcs;

    is_render_network;
    is_render_sdf;

    is_render_bezier;
    is_fill_bezier;
    is_endpoint_bezier;

    is_render_arc;
    is_fill_arc;
    is_endpoint_arc;
    arcs;

    constructor(ctx, ttf = "msyh.ttf") {
        this.init_x = 0;
        this.init_y = 0;
        this.size_x = 0;
        this.size_y = 0;

        this.mouse_x = null;
        this.mouse_y = null;

        this.last_arc_count = 0;
        this.last_bezier_count = 0;
        this.last_blob_string = "";

        this.ttf = ttf;
        this.ctx = ctx;
        this.font = null;

        this.char = "A";
        this.char_size = 256;

        this.last_arcs = null;

        this.is_render_network = true;
        this.is_render_sdf = false;

        this.is_render_bezier = true;
        this.is_fill_bezier = true;
        this.is_endpoint_bezier = true;

        this.is_render_arc = false;
        this.is_fill_arc = false;
        this.is_endpoint_arc = false;
    }

    set_mouse_down(x, y) {
        this.mouse_x = x;
        this.mouse_y = y;
        // console.warn(`mouse down: ${x}, ${y}`);
    }

    set_init_pos(x, y) {
        this.init_x = x;
        this.init_y = y;
    }

    set_init_size(x, y) {
        this.size_x = x;
        this.size_y = y;
    }

    set_render_network(is_render) {
        this.is_render_network = is_render;
    }

    set_render_sdf(is_render) {
        this.is_render_sdf = is_render;
    }

    set_render_bezier(is_render) {
        this.is_render_bezier = is_render;
    }

    set_bezier_fill(is_fill) {
        this.is_fill_bezier = is_fill;
    }

    set_bezier_endpoints(is_endpoint) {
        this.is_endpoint_bezier = is_endpoint;
    }

    set_render_arc(is_render) {
        this.is_render_arc = is_render;
    }

    set_arc_fill(is_fill) {
        this.is_fill_arc = is_fill;
    }

    set_arc_endpoints(is_endpoint) {
        this.is_endpoint_arc = is_endpoint;
    }

    set_char_size(size) {
        // if (this.char_size !== size) {
        //     delete_glyph(this.char);
        // }
        this.char_size = size;

        // if (!this.font) {
        //     this.font = this.load()
        // }
    }

    set_char(char) {
        if (this.char !== char[0]) {
            this.arcs = get_char_arc_debug(char[0])
        }
        this.char = char[0];

        // if (!this.font) {
        //     this.font = this.load()
        // }
    }

    get_char() {
        return this.char;
    }

    clear() {
        let ctx = this.ctx;
        ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    }

    draw_network_endpoints() {

        let x = this.mouse_x;
        let y = this.mouse_y;

        if (x === null || y === null) {
            return;
        }

        if (!this.is_render_network) {
            return;
        }

        if (!this.last_arcs) {
            return;
        }

        let cellSize = this.last_arcs.cell_size;


        // 计算点击位置对应的网格坐标
        x = x - this.init_x;
        y = -y + this.init_y;
        let extents = this.last_arcs.get_extents();

        x -= extents.min_x;
        y -= extents.min_y;

        let i = Math.floor(x / cellSize);
        let j = Math.floor(y / cellSize);

        if (j < 0 || j >= this.last_arcs.height_cells) {
            return;
        }

        if (i < 0 || i >= this.last_arcs.width_cells) {
            return;
        }

        // 从arcs.data中获取对应的数据
        let unitArc = this.last_arcs.get_unit_arc(i, j);


        let show_data = unitArc.data;
        if (show_data.length === 1) {
            show_data = unitArc.origin_data;
        }

        let ctx = this.ctx;
        ctx.save();
        ctx.translate(this.init_x, this.init_y);
        ctx.scale(1, -1);
        let parent_cell = unitArc.parent_cell;

        ctx.strokeStyle = 'red';
        ctx.beginPath();
        ctx.moveTo(parent_cell.min_x, parent_cell.min_y);
        ctx.lineTo(parent_cell.min_x, parent_cell.max_y);
        ctx.lineTo(parent_cell.max_x, parent_cell.max_y);
        ctx.lineTo(parent_cell.max_x, parent_cell.min_y);
        ctx.lineTo(parent_cell.min_x, parent_cell.min_y);
        ctx.stroke();

        for (let k = 0; k < show_data.length; k++) {
            // 注意，这里假设data中所有的元素都是ArcEndpoint类型的
            let endpoint = show_data[k];

            if (endpoint.d === GLYPHY_INFINITY) {
                ctx.fillStyle = 'red';
            } else if (endpoint.d === 0) {
                ctx.fillStyle = 'yellow';
            } else {
                ctx.fillStyle = 'black';
            }

            // 在端点位置画出黑点
            ctx.beginPath();
            let xy = endpoint.get_xy();
            let p = new Point(xy[0], xy[1]);
            console.log(`draw_network_endpoints: (${i}, ${j}): p = (${p.x}, ${p.y}), d = ${endpoint.d}`);
            ctx.arc(p.x, p.y, 20, 0, 2 * Math.PI);
            ctx.fill();
        }
        ctx.restore();
    }

    get_arc_count() {
        return this.last_arc_count;
    }

    get_bezier_count() {
        return this.last_bezier_count;
    }

    get_blob_string() {
        return this.last_blob_string;
    }

    draw() {

        let size = 2048;

        if (!this.char) {
            return;
        }
        // let verties = add_glyph_vertices(gi);
        // console.log(`verties = `, verties);

        // let tex_data = arcs.tex_data;
        // if (!tex_data) {
        //     throw new Error(`tex_data is null`);
        // }

        // let g = set_glyph(this.char, verties, tex_data);
        // if (!g) {
        //     throw new Error(`g is null`);
        // }

        // let scale = this.char_size * window.devicePixelRatio;
        // let m = mat4.create();
        // mat4.identity(m);
        // mat4.translate(m, m, [25.0, 120.0, 0.0]);
        // mat4.scale(m, m, [scale, scale, 1.0]);
        // g.mesh?.material?.setWorldMatrix(m);

        // this.last_arc_count = endpoints.length;
        // this.last_bezier_count = svg_endpoints.length;
        // this.last_blob_string = arcs.show;

        // console.log(`svg_paths = `, svg_paths);
        // console.log(`svg_endpoints = `, svg_endpoints);
        // console.log(`endpoints = `, endpoints);
        // console.log(`arcs = `, arcs);

        this.clear();

        // if (this.is_render_bezier) {
        //     let is_fill = this.is_fill_bezier;
        //     this.draw_svg(svg_paths, this.init_x, this.init_y, size, this.size_x, this.init_x, is_fill, "red");
        // }
        // if (this.is_endpoint_bezier) {
        //     this.draw_points(svg_endpoints, this.init_x, this.init_y, "violet");
        // }

        let is_fill = this.is_fill_arc;
        this.draw_arc(this.arcs, this.init_x, this.init_y, size, this.size_x, this.init_x, is_fill, "green", "blue");

        if (this.is_render_network) {
            this.draw_network(this.arcs, this.init_x, this.init_y);
            this.draw_network_endpoints();
        }

    }

    draw_network(arcs, x, y) {

        let ctx = this.ctx;
        let cellSize = arcs.cell_size;

        this.last_arcs = arcs;
        let extents = arcs.get_extents()
        console.log(`=========draw_network: x = ${x}, y = ${y}, extents = (${extents.min_x}, ${extents.min_y}, ${extents.max_x}, ${extents.max_y}), w * h = (${arcs.width_cells}, ${arcs.height_cells}), size = ${cellSize}`);

        // 保存 ctx 的当前状态
        ctx.save();
        ctx.translate(x, y);
        ctx.scale(1, -1);
        ctx.translate(extents.min_x, extents.min_y);
        const transform = ctx.getTransform();
        const originX = transform.e;
        const originY = transform.f;

        console.log(`origin: ${originX}, ${originY}`)
        for (let i = 0; i <= arcs.width_cells; i++) {
            let posX = i * cellSize;

            // 设置笔触样式和线宽
            ctx.strokeStyle = 'gray';
            ctx.lineWidth = 1;

            // 画竖线
            ctx.beginPath();
            ctx.moveTo(posX, 0.0);
            ctx.lineTo(posX, arcs.height_cells * cellSize);
            ctx.stroke();
        }

        for (let j = 0; j <= arcs.height_cells; j++) {
            let posY = j * cellSize;

            // 设置笔触样式和线宽
            ctx.strokeStyle = 'gray';
            ctx.lineWidth = 1;

            // 画横线
            ctx.beginPath();
            ctx.moveTo(0, posY);
            ctx.lineTo(arcs.width_cells * cellSize, posY);
            ctx.stroke();
        }

        // 设置字体大小和样式
        ctx.font = `${cellSize / 6}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillStyle = 'black';

        // 在每个网格的中心写入数字
        for (let j = 0; j < arcs.height_cells; j++) {
            for (let i = 0; i < arcs.width_cells; i++) {
                let posX = (i + 0.5) * cellSize;
                let posY = (j + 0.5) * cellSize;
                let unit = arcs.get_unit_arc(i, j);
                // console.log(`x: ${i}, y: ${j}, numpoints: ${unit.show}`);
                let text = unit.show;

                ctx.save();
                ctx.scale(1, -1);
                ctx.fillText(text, posX, -posY);  // 注意这里 y 坐标的符号
                ctx.restore();
            }
        }

        // 恢复 ctx 的状态
        ctx.restore();
    }


    draw_arc(arcs, x, y, size, w, init_x, is_fill = false, color = "green", endpoints_color = "blue") {

        let [cmds, pts] = to_arc_cmds(arcs);

        // console.log("")
        // console.log(`============== 04. 圆弧`);
        // for (let cmd_array of cmds) {
        //     for (let cmd of cmd_array) {
        //         console.log(`    ${cmd}`);
        //     }
        // }
        // console.log("")

        let cmd_s = [];
        for (let cmd_array of cmds) {
            cmd_s.push(cmd_array.join(" "));
        }

        if (this.is_render_arc) {
            this.draw_svg(cmd_s, x, y, size, w, init_x, is_fill, color);
        }

        if (this.is_endpoint_arc) {
            this.draw_points(pts, x, y, endpoints_color)
        }
    }

    draw_points(pts, x, y, color = "black") {
        this.ctx.save();
        this.ctx.translate(x, y);
        this.ctx.scale(1, -1);
        for (let pt of pts) {
            this.ctx.beginPath();
            this.ctx.arc(pt[0], pt[1], 8, 0, Math.PI * 2);
            this.ctx.fillStyle = color;
            this.ctx.fill();
        }
        this.ctx.restore();
    }

    draw_svg(path_cmds, x, y, size, w, init_x, is_fill = true, color = "red") {
        let paths = []
        for (let cmd of path_cmds) {
            let path = new Path2D(cmd)
            paths.push(path)
        }

        let path = new Path2D()
        for (let p of paths) {
            path.addPath(p);
        }

        this.ctx.save(); // 保存当前的上下文状态
        this.ctx.translate(x, y);
        this.ctx.scale(1, -1);
        if (is_fill) {
            this.ctx.fillStyle = color;
            this.ctx.fill(path);
        } else {
            this.ctx.strokeStyle = color;
            this.ctx.stroke(path);
        }
        this.ctx.restore();

        x += size;
        if (x > w) {
            x = init_x;
            y += size;
        }
    }

    // async load() {
    //     return new Promise < opentype.Font > ((resolve, reject) => {
    //         opentype.load(`font/${this.ttf}`, (err, font) => {
    //             if (err || !font) {
    //                 reject(err || new Error('Font could not be loaded.'));
    //             } else {
    //                 resolve(font);
    //             }
    //         });
    //     });
    // }
}



const to_arc_cmds = (
    arcs
) => {
    let cmd = []
    let cmd_array = []
    let current_point = null;
    let pts = [];

    let len = arcs.get_endpoints_len();
    for (let i = 0; i < len; i++) {
        let endpoint = arcs.get_endpoint(i);
        let xy = endpoint.get_xy()
        let p = new Point(xy[0], xy[1]);
        pts.push([p.x, p.y]);

        if (endpoint.d === GLYPHY_INFINITY) {
            if (!current_point || !p.equals(current_point)) {
                if (cmd.length > 0) {
                    cmd_array.push(cmd);
                    cmd = []
                }
                cmd.push(` M ${p.x}, ${p.y}`)
                current_point = p;
            }
        } else if (endpoint.d === 0) {
            assert(current_point !== null);
            if (current_point && !p.equals(current_point)) {
                cmd.push(` L ${p.x}, ${p.y}`)
                current_point = p;
            }
        } else {
            assert(current_point !== null);
            if (current_point && !p.equals(current_point)) {
                let arc = new Arc(current_point, p, endpoint.d);
                let center = arc.center();
                let radius = arc.radius();
                let start_v = current_point.sub_point(center);
                let start_angle = start_v.angle();

                let end_v = p.sub_point(center);
                let end_angle = end_v.angle();

                // 大于0，顺时针绘制
                let cross = start_v.cross(end_v);

                cmd.push(arcToSvgA(
                    center.x, center.y, radius,
                    start_angle, end_angle, cross < 0));

                current_point = p;
            }
        }
    }
    if (cmd.length > 0) {
        cmd_array.push(cmd);
        cmd = []
    }

    return [cmd_array, pts];
};

const arcToSvgA = (x, y, radius, startAngle, endAngle, anticlockwise) => {
    // 计算圆弧结束点坐标
    let endX = x + radius * Math.cos(endAngle);
    let endY = y + radius * Math.sin(endAngle);

    // large-arc-flag 的值为 0 或 1，决定了弧线是大于还是小于或等于 180 度
    let largeArcFlag = '0' // endAngle - startAngle <= Math.PI ? '0' : '1';

    // sweep-flag 的值为 0 或 1，决定了弧线是顺时针还是逆时针方向
    let sweepFlag = anticlockwise ? '0' : '1';

    // 返回 SVG "A" 命令参数
    return `A ${radius} ${radius} 0 ${largeArcFlag} ${sweepFlag} ${endX} ${endY}`;
}




export class Point {
    x;
    y;

    constructor(x_ = 0.0, y_ = 0.0) {
        this.x = x_;
        this.y = y_;
    }

    /**
     * Point 转 向量
     */
    // into_vector() {
    //     return new Vector(this.x, this.y);
    // }

    /**
     * 通过向量 新建点
     */
    // static from_vector(v: Vector) {
    //     return new Point(v.x, v.y);
    // }

    /**
     * 克隆 点
     */
    clone() {
        return new Point(this.x, this.y);
    }

    /**
     * this 是否等于 p
     */
    equals(p) {
        return float_equals(this.x, p.x) && float_equals(this.y, p.y);
    }

    /**
     * 点 加 向量
     */
    add_vector(v) {
        return new Point(this.x + v.x, this.y + v.y);
    }

    /**
     * 点 减 向量
     */
    sub_vector(v) {
        return new Point(this.x - v.x, this.y - v.y);
    }

    /**
     * 点 减 点
     */
    sub_point(p) {
        return new Point(this.x - p.x, this.y - p.y);
    }

    /**
     * 点 加向量 赋值
     */
    add_assign(v) {
        this.x += v.x;
        this.y += v.y;
        return this;
    }

    /**
     * 点 减向量 赋值
     */
    sub_assign(v) {
        this.x -= v.x;
        this.y -= v.y;
        return this;
    }

    /**
     * 取中点
     */
    midpoint(p) {
        return new Point((this.x + p.x) / 2.0, (this.y + p.y) / 2.0);
    }

    /**
     * TODO
     */
    bisector(p) {
        let d = p.sub_point(this);
        return new Line(d.x * 2, d.y * 2, p.into_vector().dot(d) + this.into_vector().dot(d));
    }

    /**
     * 到 点p的距离的平方
     */
    squared_distance_to_point(p) {
        let v = this.sub_point(p)
        return v.len2();
    }

    /**
     * 到 点p的距离
     */
    distance_to_point(p) {
        return Math.sqrt(this.squared_distance_to_point(p));
    }

    /**
     * 是否 无穷
     */
    is_infinite() {
        return (this.x === Infinity || this.x === -Infinity)
            && (this.y === Infinity || this.y === -Infinity);
    }

    /**
     * 线性 插值
     */
    lerp(a, p) {
        if (a == 0) {
            return this;
        }
        if (a == 1.0) {
            return p;
        }

        return new Point((1 - a) * this.x + a * p.x, (1 - a) * this.y + a * p.y);
    }

    angle() {
        return Math.atan2(this.y, this.x);
    }

    /**
     * 向量 叉积
     */
    cross(other) {
        return this.x * other.y - this.y * other.x;
    }

     /**
     * 垂直 向量
     */
     ortho() {
        return new Point(-this.y, this.x);
    }

     /**
     * 向量 数量积
     */
     scale(s) {
        return new Point(this.x * s, this.y * s);
    }

    /**
     * 到 线l的最短距离
     */
    // shortest_distance_to_line(l): SignedVector {
    //     return l.sub(this).neg();
    // }

    /**
     * 向量 点积
     */
    dot(v) {
        return this.x * v.x + this.y * v.y;
    }


    /**
     * 向量 长度的平方
     */
    len2() {
        return this.dot(this)
    }

    /**
     * 向量 长度
     */
    len() {
        return Math.sqrt(this.len2());
    }
}

export const float_equals = (
    f1,
    f2,
    error = GLYPHY_EPSILON
) => {
    return Math.abs(f1 - f2) < error;
};
// tan( 2 * atan(d) )
export const tan2atan = (d) => {
    return 2 * d / (1 - d * d);
}

// sin( 2 * atan(d) )
export const sin2atan = (d) => {
    return 2 * d / (1 + d * d);
}

/**
 * 断言：参数为false时，抛异常
 */
export const assert = (arg, msg = "") => {
    if (!arg) {
        throw new Error(`Assertion failed: msg = ${msg}`);
    }
};

export class Arc {
    p0;
    p1;
    d;

    /**
     * 构造函数
     */
    constructor(p0, p1, d) {
        this.p0 = p0;
        this.p1 = p1;
        this.d = d;
    }

    /**
     * 从三个点 构造 圆弧
     * @param p0 起点
     * @param p1 终点
     * @param pm 中间点
     * @param complement 是否补弧
     */
    static from_points(p0, p1, pm, complement) {
        let arc = new Arc(p0, p1, 0.0);
        if (p0 != pm && p1 != pm) {
            let v = p1.sub_point(pm);
            let u = p0.sub_point(pm);
            arc.d = Math.tan(((v.angle() - u.angle()) / 2) - (complement ? 0 : Math.PI / 2));
        }
        return arc
    }

    /**
     * 从圆心、半径、起始角度、终止角度 构造 圆弧
     * @param center 圆心
     * @param radius 半径
     * @param a0 起始角度
     * @param a1 终止角度
     * @param complement 是否补弧
     */
    static from_center_radius_angle(center, radius, a0, a1, complement) {

        let p0 = center.add_vector(new Vector(Math.cos(a0), Math.sin(a0)).scale(radius));
        let p1 = center.add_vector(new Vector(Math.cos(a1), Math.sin(a1)).scale(radius));
        let d = Math.tan(((a1 - a0) / 4) - (complement ? 0 : Math.PI / 2));
        return new Arc(p0, p1, d);
    }

    to_svg_command() {
        const start_point = this.p0;
        const end_point = this.p1;

        const radius = this.radius();
        const center = this.center();

        const start_angle = Math.atan2(start_point.y - center.y, start_point.x - center.x);
        const end_angle = Math.atan2(end_point.y - center.y, end_point.x - center.x);

        // large-arc-flag 是一个布尔值（0 或 1），表示是否选择较大的弧（1）或较小的弧（0）
        const large_arc_flag = Math.abs(end_angle - start_angle) > Math.PI ? 1 : 0;

        // sweep-flag 是一个布尔值（0 或 1），表示弧是否按顺时针（1）或逆时针（0）方向绘制。
        const sweep_flag = this.d > 0 ? 1 : 0;

        // x-axis-rotation 是椭圆的 x 轴与水平方向的夹角，单位为度。
        // A rx ry x-axis-rotation large-arc-flag sweep-flag x y
        const arc_command = `A ${radius} ${radius} 0 ${large_arc_flag} ${sweep_flag} ${end_point.x} ${end_point.y}`;

        return arc_command;
    }

    /**
     * 克隆
     */
    clone() {
        return new Arc(this.p0, this.p1, this.d);
    }

    /**
     * 相等
     */
    equals(a) {
        return this.p0.equals(a.p0) && this.p1.equals(a.p1) && float_equals(this.d, a.d);
    }

    /**
     * 减去 点
     */
    sub(p) {
        if (Math.abs(this.d) < 1e-5) {
            const arc_segment = new Segment(this.p0, this.p1);
            return arc_segment.sub(p);
        }

        if (this.wedge_contains_point(p)) {
            const difference = this.center()
                .sub_point(p)
                .normalized()
                .scale(Math.abs(p.distance_to_point(this.center()) - this.radius()));

            let d = xor(this.d < 0, p.sub_point(this.center()).len() < this.radius());
            return SignedVector.from_vector(difference, d);
        }

        const d0 = p.squared_distance_to_point(this.p0);
        const d1 = p.squared_distance_to_point(this.p1);

        const other_arc = new Arc(this.p0, this.p1, (1.0 + this.d) / (1.0 - this.d));
        const normal = this.center().sub_point(d0 < d1 ? this.p0 : this.p1);

        if (normal.len() === 0) {
            return SignedVector.from_vector(new Vector(0, 0), true);
        }

        let min_p = d0 < d1 ? this.p0 : this.p1;
        let l = new Line(normal.x, normal.y, normal.dot(min_p.into_vector()));
        return SignedVector.from_vector(l.sub(p), !other_arc.wedge_contains_point(p));
    }

    /**
     * 计算圆弧的半径
     * @returns {number} 圆弧半径
     */
    radius() {
        return Math.abs((this.p1.sub_point(this.p0)).len() / (2 * sin2atan(this.d)));
    }

    /**
     * 计算圆弧的圆心
     * @returns {Point} 圆弧的圆心
     */
    center() {
        return (this.p0.midpoint(this.p1)).add_vector((this.p1.sub_point(this.p0)).ortho().scale(1 / (2 * tan2atan(this.d))));
    }

    /**
     * 计算圆弧 的 切线向量对
     * 
     * 圆弧切线，就是 圆弧端点在圆上的切线
     * 
     * 切线向量 和 圆心到圆弧端点的向量 垂直
     * 
     * 算法：以 半弦 为基准，计算切线向量
     * 
     * 圆心 为 O，起点是A，终点是B
     * 
     * 以 A 为圆心，半弦长 为半径，画一个圆，和 AO 相交于 点 C
     * 
     * |AC| = |AB| / 2
     * 
     * 将有向线段 AC 分解到 半弦 和 半弦 垂线上，分别得到下面的 result_dp 和 pp
     */
    tangents() {
        const dp = (this.p1.sub_point(this.p0)).scale(0.5);
        const pp = dp.ortho().scale(-sin2atan(this.d));

        const result_dp = dp.scale(cos2atan(this.d));

        return {
            first: result_dp.add(pp),  // 起点 切线向量，注：没有单位化
            second: result_dp.sub(pp), // 终点 切线向量，注：没有单位化
        };
    }

    /**
     * 将圆弧近似为贝塞尔曲线
     */
    approximate_bezier(error) {
        const dp = this.p1.sub_point(this.p0);
        const pp = dp.ortho();

        if (error) {
            error.value = dp.len() * Math.pow(Math.abs(this.d), 5) / (54 * (1 + this.d * this.d));
        }

        const result_dp = dp.scale((1 - this.d * this.d) / 3);
        const result_pp = pp.scale(2 * this.d / 3);

        const p0s = this.p0.add_vector(result_dp).sub_vector(result_pp);
        const p1s = this.p1.sub_vector(result_dp).sub_vector(result_pp);

        return new Bezier(this.p0, p0s, p1s, this.p1);
    }

    /**
     * 判断 p 是否包含在 圆弧对扇形的夹角内。
     * 
     * 包括 圆弧边缘 的 线
     * 
     */
    wedge_contains_point(p) {
        const t = this.tangents();

        if (Math.abs(this.d) <= 1) {
            // 小圆弧，夹角 小于等于 PI
            // 在 夹角内，意味着 下面两者 同时成立：
            //     向量 <P0, P> 和 起点切线 成 锐角
            //     向量 <P1, P> 和 终点切线 是 钝角
            return (p.sub_point(this.p0)).dot(t.first) >= 0 && (p.sub_point(this.p1)).dot(t.second) <= 0;
        } else {
            // 大圆弧，夹角 大于 PI
            // 如果 点 在 小圆弧 内，那么：下面两者 同时成立
            //     向量 <P0, P> 和 起点切线 成 钝角
            //     向量 <P1, P> 和 终点切线 是 锐角
            // 所以这里要 取反
            return (p.sub_point(this.p0)).dot(t.first) >= 0 || (p.sub_point(this.p1)).dot(t.second) <= 0;
        }
    }

    /**
     * 计算点到圆弧的距离
     */
    distance_to_point(p) {
        if (Math.abs(this.d) < 1e-5) {
            // d = 0, 当 线段 处理
            const arc_segment = new Segment(this.p0, this.p1);
            return arc_segment.distance_to_point(p);
        }

        const difference = this.sub(p);

        if (this.wedge_contains_point(p) && Math.abs(this.d) > 1e-5) {
            // 在 夹角内

            // 距离的绝对值 就是 |点到圆心的距离 - 半径|
            // 符号，看 difference 的 neggative
            return Math.abs(p.distance_to_point(this.center()) - this.radius()) * (difference.negative ? -1 : 1);
        }

        const d1 = p.squared_distance_to_point(this.p0);
        const d2 = p.squared_distance_to_point(this.p1);

        return (d1 < d2 ? Math.sqrt(d1) : Math.sqrt(d2)) * (difference.negative ? -1 : 1);
    }

    /**
     * 计算点到圆弧的平方距离
     */
    squared_distance_to_point(p) {
        if (Math.abs(this.d) < 1e-5) {
            const arc_segment = new Segment(this.p0, this.p1);
            // 点 到 线段 的 距离 的 平方
            return arc_segment.squared_distance_to_point(p);
        }

        if (this.wedge_contains_point(p) && Math.abs(this.d) > 1e-5) {
            // 在圆弧的 夹角 里面，sdf = 点到圆心的距离 - 半径
            const answer = p.distance_to_point(this.center()) - this.radius();
            return answer * answer;
        }

        // 在 夹角外，就是 点 到 啷个端点距离的 最小值
        const d1 = p.squared_distance_to_point(this.p0);
        const d2 = p.squared_distance_to_point(this.p1);

        return (d1 < d2 ? d1 : d2);
    }

    /**
     * 计算点到圆弧的扩展距离
     */
    extended_dist(p) {
        // m 是 P0 P1 的 中点
        const m = this.p0.lerp(0.5, this.p1);
        
        // dp 是 向量 <P0, P1>
        const dp = this.p1.sub_point(this.p0);
        
        // pp 是 dp 的 正交向量，逆时针
        const pp = dp.ortho();

        // d2 是 圆弧的 圆心角一半 的正切
        const d2 = tan2atan(this.d);

        if (p.sub_point(m).dot(this.p1.sub_point(m)) < 0) {
            // 如果 <M, P> 和 <P1, P> 夹角 为 钝角
            // 代表 P 在 直径为 <M, P1> 的 圆内

            // <P0, P> 与 N1 方向的 投影
            // N1 = pp + dp * tan(angle / 2)
            return (p.sub_point(this.p0)).dot( (pp.add(dp.scale(d2))).normalized() );
        } else {
            // <P1, P> 与 N2 的 点积
            // N2 = pp - dp * tan(angle / 2)
            return (p.sub_point(this.p1)).dot( (pp.sub(dp.scale(d2))).normalized() );
        }
    }

    /**
     * 计算圆弧的包围盒
     * @returns {Array<Point>} 包围盒的顶点数组
     */
    extents(e) {
        e.clear()
        e.add(this.p0);
        e.add(this.p1);

        const c = this.center();
        const r = this.radius();
        const p = [
            c.add_vector(new Vector(-1, 0).scale(r)),
            c.add_vector(new Vector(1, 0).scale(r)),
            c.add_vector(new Vector(0, -1).scale(r)),
            c.add_vector(new Vector(0, 1).scale(r)),
        ];

        for (let i = 0; i < 4; i++) {
            if (this.wedge_contains_point(p[i])) {
                e.add(p[i]);
            }
        }
    }
}