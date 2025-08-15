
let fs = require("fs");


/**
 * argv: 
 *    [0]: node.exe的路径
 *    [1]: *.js 执行的js的路径
 *    [2..]: 其他参数
 */

// 当前目录，一般是项目地址
let cwd = process.cwd();

var dir = process.argv[2] || "pkg";
var name = process.argv[3] || "gui";
var cfgPath = process.argv[4] || "temp/cfg.txt";
var wasmName = `${name}_bg`;

let outDir;
let data = fs.readFileSync(cfgPath, {encoding:"utf8"});
let datas = data.split("=");
if (datas.length == 2) {
	let d = datas[1].trim();
	if (d !== "") {
		outDir = d;
	}
	
}

let in_wasm_path = `${dir}/${wasmName}.wasm`;
let in_wasm_js_path = `${dir}/${name}.js`;
let out_wasm_path = `${dir}/${name}.wasm`;
let out_wasm_js_path = `${dir}/${name}.wasm.ts`;

fs.readFile(in_wasm_js_path, {encoding:"utf8"}, (err, data) => {
	if(!err) {
		data = data.replace(`import.meta.url`, '""');
		data = data.replace(/(from '[.a-zA-Z0-9/]*)pi_hal\-[a-z0-9]*/g, function(_match, p0) {return p0 + 'pi_hal'});
		data = data.replace(/(from '[.a-zA-Z0-9/]*)pi_bon_decode\-[a-z0-9]*/g, function(_match, p0) {return p0 + 'pi_bon_decode'});
		data = data.replace(/from\s+'(.+?)\.js'/g,  "from '$1'");
		data = data.replace(/getObject\(arg0\)\sinstanceof\sWindow/g, "true");
		data = data.replace(/getObject\(arg0\)\sinstanceof\sCanvasRenderingContext2D/g, "true");
		data = data.replace(/getObject\(arg0\)\sinstanceof\sHTMLCanvasElement/g, "true");

		data = data.replace(/=== 0 \? undefined/g, "=== 0 ? null");

		// wasm崩溃时，通知，以便外部做进一步处理。（因为目前浏览器对wasm的支持不稳定，在某些版本的chrome内核，会出现不同程度的bug，通知出去，使得外部可以采用一些备用方案，如：总是为其下载固定版本的chrome内核）
		data = data.replace(
`
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
`,
`
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
		let str = getStringFromWasm0(arg0, arg1);
		window._$pi?._wasmThrow?.(str);
        throw new Error(str);
    };
`
		)

		data = data.replace(
`    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;`,
`    const r = await __wbg_load(await module_or_path, imports);

    let ret = __wbg_finalize_init(r.instance, r.module);
	
	if(module.postRun) {
		module.postRun();
	}

    return ret;
}

Promise.resolve().then(() => {
	window["__wasm"] = __wbg_init(module.wasmModule);
})`);
		// data = data.replace(`Module["noExitRuntime"]=true;run();`, `Module["noExitRuntime"] = true;
		// //PI_START
		// run();
		// window.Module = Module;
		// // run();
		// //PI_END
		// `);

		data = data.replace("function getObject(idx) { return heap[idx]; }", "function getObject(idx) { let result = heap[idx]; if (result === undefined) { return null } else { return result }; }");

		fs.writeFile(out_wasm_js_path, data, {encoding:"utf8"}, (err) => {
			if(err) {
				console.log("写文件失败！！", JSON.stringify(err));
			}
		});

		if (outDir) {
			fs.writeFile(`${outDir}/${name}.wasm.ts`, data, (err) => {
				if(err) {
					console.log("写文件失败！！", JSON.stringify(err));
				}
			})
		}
	} else {
		console.log("读文件失败！！", JSON.stringify(err));
	}
});

fs.readFile(in_wasm_path, (err, data) => {
	if(!err) {
		fs.writeFile(out_wasm_path, data, (err) => {
			if(err) {
				console.log("写文件失败！！", JSON.stringify(err));
			}
		})

		console.log("wasm拷贝到:", outDir);
		if (outDir) {
			fs.writeFile(`${outDir}/${name}.wasm`, data, (err) => {
				if(err) {
					console.log("写文件失败！！", JSON.stringify(err));
				}
			})
		}
	} else {
		console.log("读文件失败！！", JSON.stringify(err));
	}
});