#!/usr/bin/env python3
"""Import literal tables from the pinned truth checkout into reviewable native maps.
Offline maintainer tool only: the application never reads Rust source. Unknown
source shapes fail rather than guessing. Existing source snapshots stay intact.
"""
import argparse, hashlib, json, re, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
PIN = 'c78a541d1fb09fcefd448c83b097d4f9b815fce3'
ORDER = ['Th06','Th06NC','Th07','Th08','Th09','Th095','Th10','Alcostg','Th11','Th12','Th125','Th128','Th13','Th14','Th143','Th15','Th16','Th165','Th17','Th18','Th185','Th19','Th20']
IDS = [6,61,7,8,9,95,10,103,11,12,125,128,13,14,143,15,16,165,17,18,185,19,20]
GAMES = dict(zip(IDS, ORDER))
SETS = {'msg':[6,7,8,9,10,11,12,128,13,14,143,15,16,165,17,18,185,19,20], 'ending':[10,11,12,128,13,14,15,16,17,18,19,20], 'std':[6,7,8,9,95,10,103,11,12,125,128,13,14,143,15,16,165,17,18,185,19,20]}
EXT={'msg':'msgm','ending':'endm','std':'stdm'}
def sections(path):
    out={}; section=''
    if not path.exists(): return out
    for line in path.read_text().splitlines()[1:]:
        line=line.split('#')[0].strip()
        if not line: continue
        if line.startswith('!'): section=line;out.setdefault(section,[]);continue
        match=re.fullmatch(r'(-?\d+)\s*(.*)',line)
        if match:out.setdefault(section,[]).append((int(match[1]),match[2]))
    return out

def table(source, name, game):
    block=re.search(r'static '+name+r':.*?ins: &\[(.*?)\],\s*var:',source,re.S)
    assert block, name
    result={}; intrinsic={}
    for raw in block[1].splitlines():
        line=raw.split('//')[0].strip()
        if not line:continue
        m=re.fullmatch(r'\((\w+),\s*(\d+),\s*(.*?)\),',line)
        assert m, line
        version,op,value=m[1],int(m[2]),m[3]
        if ORDER.index(version)>ORDER.index(GAMES[game]):continue
        if value=='None':result.pop(op,None);intrinsic.pop(op,None);continue
        v=re.fullmatch(r'Some\(\((r#".*"#|"(?:\\.|[^"\\])*")\s*,\s*(None|Some\(IKind::(\w+)\))\)\)',value)
        assert v,value
        text=v[1];result[op]=text[3:-2] if text.startswith('r#') else json.loads(text)
        if v[3]:intrinsic[op]={'Jmp':'Jmp()','InterruptLabel':'Interrupt()'}[v[3]]
    return result,intrinsic

def main():
    parser=argparse.ArgumentParser();parser.add_argument('truth_checkout',type=Path);args=parser.parse_args();truth=args.truth_checkout
    assert subprocess.check_output(['git','-C',str(truth),'rev-parse','HEAD'],text=True).strip()==PIN
    dest=ROOT/'src-tauri/assets/maps/truth/defaults';dest.mkdir(exist_ok=True)
    zero=ROOT/'src-tauri/assets/maps/truth/zero318';old=ROOT/'src-tauri/assets/maps'
    records=[];entries=[]
    for kind,games in SETS.items():
        ext=EXT[kind];src=truth/'src/core_mapfiles'/('end.rs' if kind=='ending' else kind+'.rs');source=src.read_text()
        for game in games:
            name=('END_10_20' if kind=='ending' else ('MSG_06_09' if game in [6,7,8,9] else 'MSG_10_20') if kind=='msg' else 'STD_06' if game==6 else 'STD_07_09' if game in [7,8,9] else 'STD_095_20')
            sigs,intrinsics=table(source,name,game)
            zero_index=dict(sections(zero/('any.'+ext)).get('!game_files',[]));zero_path=zero/zero_index[game] if game in zero_index else None
            z=sections(zero_path) if zero_path else {}
            # Only annotate instructions actually supported by this game's core table.
            for op,sig in z.get('!ins_signatures',[]):
                if op in sigs:sigs[op]=sig
            aliases=[(op,n) for op,n in z.get('!ins_names',[]) if op in sigs]
            if kind=='ending': legacy=old/('th20.endm' if game==20 else 'th10.endm')
            elif kind=='msg' and game in [143,165]: legacy=truth/'map/th11-scene.msgm'
            elif kind=='msg': legacy=old/('th10.msgm' if game==10 else f'th{game}.msgm' if game in [19,20] else 'th11.msgm') if game not in [6,7,8,9] else None
            else: legacy=old/'th14.stdm' if ORDER.index(GAMES[game])>=ORDER.index('Th14') else None
            if legacy is None:
                index=dict(sections(truth/'map'/('any.'+ext)).get('!game_files',[]));legacy=truth/'map'/index[game] if game in index else None
            if legacy:
                aliases.extend((op,n) for op,n in sections(legacy).get('!ins_names',[]) if op in sigs)
            # Later names are preferred, while prior names remain valid aliases.
            rows=['!endmap' if kind=='ending' else '!'+kind+'map','',f'# Studio {kind}, th{game}; generated from pinned truth/TouhouMaps.','!ins_names']
            rows += [f'{op} {n}' for op,n in aliases]
            rows += ['','!ins_signatures']+[f'{op} {sig}'.rstrip() for op,sig in sorted(sigs.items())]
            if intrinsics:rows += ['','!ins_intrinsics']+[f'{op} {v}' for op,v in sorted(intrinsics.items())]
            for section,values in z.items():
                if section.startswith('!enum('): rows += ['',section]+[f'{k} {v}' for k,v in values]
            content='\n'.join(rows)+'\n';filename=f'{kind}-{game}.{ext}';(dest/filename).write_text(content)
            records.append({'file':filename,'truthSource':str(src.relative_to(truth)),'sourceSha256':hashlib.sha256(src.read_bytes()).hexdigest(),'zero318Map':zero_path.name if zero_path else None,'sha256':hashlib.sha256(content.encode()).hexdigest()})
            variant={'msg':'Msg','ending':'Ending','std':'Std'}[kind]
            entries.append(f'    ({game}, ScriptKind::{variant}, "{filename}", include_str!("../../../src-tauri/assets/maps/truth/defaults/{filename}")),')
    (dest/'PROVENANCE.json').write_text(json.dumps({'truthCommit':PIN,'mapsCommit':'0673f4221f5e975842162e07cd7b765fc7570ebd','generator':'scripts/import-truth-maps.py','licenses':['Apache-2.0','Unlicense'],'files':records},indent=2)+'\n')
    shutil_license=truth/'LICENSE';(dest/'LICENSE.truth').write_bytes(shutil_license.read_bytes())
    (ROOT/'crates/script-check/src/bundled_maps.rs').write_text('// Generated by scripts/import-truth-maps.py; do not hand edit.\nuse crate::ScriptKind;\npub const MAPS: &[(u32, ScriptKind, &str, &str)] = &[\n'+'\n'.join(entries)+'\n];\n')
    print(f'Imported {len(records)} versioned native maps')
if __name__=='__main__':main()
