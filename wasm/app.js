// import { SdfContext } from "./sdf/draw_sdf";
// import { set_gl } from "./sdf/glyph";
import { DrawText } from "./draw_text.js"

import init from './pkg/pi_sdf.js';

init().then(() => {
    if (document === null) {
        alert("Failed to get document!");
        return;
    }

    let c = document.getElementById('sdf-canvas');
    if (!c) {
        alert("Failed to get sdf-canvas!");
        return;
    }
    // let sdfContext = new SdfContext(c);
    // set_gl(sdfContext.gl);
    // sdfContext.draw();

    c = document.getElementById('font-canvas');
    if (!c) {
        alert("Failed to get font-canvas!");
        return;
    }
    let fontCanvas = c;
    const fontContext = fontCanvas.getContext('2d');
    if (!fontContext) {
        alert("Failed to get font-canvas context!");
        return;
    }

    fontCanvas.addEventListener('mousedown', (event) => {
        let rect = fontCanvas.getBoundingClientRect();
        let x = event.clientX - rect.left;
        let y = event.clientY - rect.top;

        if (dt) {
            dt.set_mouse_down(x, y);
            dt.draw();
            afterDraw();
        }
    });

    const arcCountElement = document.getElementById('arc_count');

    const setArcCount = (value) => {
        if (arcCountElement) {
            arcCountElement.innerHTML = value.toString();
        }
    }

    const bezierCountElement = document.getElementById('bezier_count');
    const setBezierCount = (value) => {
        if (bezierCountElement) {
            bezierCountElement.innerHTML = value.toString();
        }
    }

    const dataTexturePixelsElement = document.getElementById('data_texture_pixels');
    const setDataTexturePixel = (show) => {
        if (dataTexturePixelsElement) {
            dataTexturePixelsElement.innerHTML = show;
        }
    }

    let dt = new DrawText(fontContext, "msyh.ttf");

    const afterDraw = () => {
        setTimeout(() => {
            // sdfContext.setChar(dt.get_char());

            // setArcCount(dt.get_arc_count());
            // setBezierCount(dt.get_bezier_count());
            // setDataTexturePixel(dt.get_blob_string());
        }, 1);
    };

    const debugElement = document.getElementById('debug');
    if (debugElement) {
        debugElement.addEventListener('change', function () {
            const debugDiv = document.getElementById("debug_canvas");
            if (debugDiv) {
                debugDiv.style.display = debugElement.checked ? "block" : "none";
            }
        });
    }

    const charElement = document.getElementById('char');
    const charValue = charElement ? charElement.value : "A";
    dt.set_char(charValue);
    charElement.addEventListener('input', function (event) {
        let target = event.target;
        dt.set_char(target.value);
        dt.draw();

        afterDraw();
    });

    const convertInputToNumber = (inputValue) => {
        if (!/^\d+$/.test(inputValue)) {
            console.warn(`警告: 大小设置，输入不完全是数字，value = ${inputValue}`);
            return -1;
        }

        return Number(inputValue);
    }

    const charSizeElement = document.getElementById('char_size');
    const charSizeValue = charSizeElement ? charSizeElement.value : "64";
    let size = convertInputToNumber(charSizeValue);
    if (size > 0) {
        dt.set_char_size(size);
    }
    charSizeElement.addEventListener('input', function (event) {
        let target = event.target;
        let size = convertInputToNumber(target.value);
        if (size > 0) {
            dt.set_char_size(size);
        }
        dt.draw();
        afterDraw();
    });

    const bezierRenderElement = document.getElementById('isBezierRender');
    const isBezierRender = bezierRenderElement ? bezierRenderElement.checked : false;
    dt.set_render_bezier(isBezierRender);
    bezierRenderElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_render_bezier(target.checked);
        dt.draw();

        afterDraw();
    });

    const bezierFillElement = document.getElementById('bezierFill');
    const isBezierFill = bezierFillElement ? bezierFillElement.checked : false;
    dt.set_bezier_fill(isBezierFill);
    bezierFillElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_bezier_fill(target.checked);
        dt.draw();

        afterDraw();

    });

    const bezierStrokeElement = document.getElementById('bezierStroke');
    bezierStrokeElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_bezier_fill(!target.checked);
        dt.draw();

        afterDraw();
    });

    const bezierEndpointsElement = document.getElementById('bezierEndpoints');
    const bezierEndpoints = bezierEndpointsElement ? bezierEndpointsElement.checked : false;
    dt.set_bezier_endpoints(bezierEndpoints);
    bezierEndpointsElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_bezier_endpoints(target.checked);
        dt.draw();

        afterDraw();
    });

    const arcRenderElement = document.getElementById('isArcRender');
    const isArcRender = arcRenderElement ? arcRenderElement.checked : false;
    dt.set_render_arc(isArcRender);
    arcRenderElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_render_arc(target.checked);
        dt.draw();

        afterDraw();
    });

    const arcFillElement = document.getElementById('arcFill');
    const isArcFill = arcFillElement ? arcFillElement.checked : false;
    dt.set_arc_fill(isArcFill);
    arcFillElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_arc_fill(target.checked);
        dt.draw();

        afterDraw();
    });

    const arcStrokeElement = document.getElementById('arcStroke');
    arcStrokeElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_arc_fill(!target.checked);
        dt.draw();

        afterDraw();
    });

    const arcEndpointsElement = document.getElementById('arcEndpoints');
    const arcEndpoints = arcEndpointsElement ? arcEndpointsElement.checked : false;
    dt.set_arc_endpoints(arcEndpoints);
    arcEndpointsElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_arc_endpoints(target.checked);
        dt.draw();

        afterDraw();
    });

    const networkRenderElement = document.getElementById('grid');
    const isNetworkRender = networkRenderElement ? networkRenderElement.checked : false;
    dt.set_render_network(isNetworkRender);
    networkRenderElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_render_network(target.checked);
        dt.draw();

        afterDraw();
    });

    const sdfRenderElement = document.getElementById('isSDFRender');
    const isSDFRender = sdfRenderElement ? sdfRenderElement.checked : false;
    dt.set_render_sdf(isSDFRender);
    networkRenderElement.addEventListener('change', function (event) {
        let target = event.target;
        dt.set_render_sdf(target.checked);
        dt.draw();

        afterDraw();
    });

    dt.set_init_pos(300, 2100);
    dt.set_init_size(fontCanvas.width, fontCanvas.height);
    dt.draw();

    afterDraw();
});
