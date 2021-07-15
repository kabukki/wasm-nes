import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import copy from 'rollup-plugin-copy';

export default {
    input: 'index.ts',
    output: {
        file: 'dist/index.js',
        format: 'esm',
    },
    plugins: [
        nodeResolve(),
        typescript(),
        copy({
            targets: [
                { src: 'pkg/*.wasm', dest: 'dist/' },
            ],
        }),
    ],
};
