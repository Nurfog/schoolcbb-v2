#!/usr/bin/env python3
"""Reconstruye kb-chunks.json desde los textos fuente en docs/curriculo/"""
import json, os, re

BASE = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..'))
CURRICULO = os.path.join(BASE, 'docs', 'curriculo')
RUST_DATA = os.path.join(BASE, 'rust', 'data')

def add_chunk(chunks, text, source, category, chunk_id):
    for p in re.split(r'\n\n+', text):
        p = p.strip()
        if len(p) < 50: continue
        if len(p) > 2000:
            for s in re.split(r'(?<=[.!?])\s+', p):
                s = s.strip()
                if len(s) > 50:
                    chunks.append({'id': chunk_id, 'text': s, 'source': source, 'category': category})
                    chunk_id += 1
        else:
            chunks.append({'id': chunk_id, 'text': p, 'source': source, 'category': category})
            chunk_id += 1
    return chunk_id

chunks = []
chunk_id = 0

for root, dirs, files in os.walk(CURRICULO):
    for f in sorted(files):
        if not f.endswith('.txt'): continue
        fp = os.path.join(root, f)
        cat = os.path.relpath(root, CURRICULO).split(os.sep)[0] if root != CURRICULO else 'root'
        try:
            with open(fp, 'r') as fh:
                chunk_id = add_chunk(chunks, fh.read(), f, cat, chunk_id)
        except: pass

output = {'chunks': chunks, 'total': len(chunks), 'total_chars': sum(len(c['text']) for c in chunks)}
outpath = os.path.join(CURRICULO, 'kb-chunks.json')
with open(outpath, 'w') as f:
    json.dump(output, f, ensure_ascii=False)

import shutil
shutil.copy2(outpath, os.path.join(RUST_DATA, 'kb-chunks.json'))

print(f"{output['total']} chunks, {output['total_chars']} chars -> {outpath} + rust/data/")
