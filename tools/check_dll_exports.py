"""Check example DLL architecture and lifecycle export names without loading them."""
import argparse,json,struct
from pathlib import Path
def exports(path):
    data=path.read_bytes()
    def u16(at):return struct.unpack_from('<H',data,at)[0]
    def u32(at):return struct.unpack_from('<I',data,at)[0]
    assert data[:2]==b'MZ',path
    pe=u32(0x3c);assert data[pe:pe+4]==b'PE\0\0',path
    machine=u16(pe+4);count=u16(pe+6);optional=pe+24;magic=u16(optional)
    assert magic in (0x10b,0x20b),magic
    sections=optional+u16(pe+20)
    def offset(rva):
        for index in range(count):
            at=sections+index*40
            size,virtual,raw_size,raw=struct.unpack_from('<IIII',data,at+8)
            if virtual<=rva<virtual+max(size,raw_size):return raw+rva-virtual
        raise ValueError(f'unmapped RVA {rva:x}')
    table=offset(u32(optional+(96 if magic==0x10b else 112)))
    names=offset(u32(table+32));result=[]
    for index in range(u32(table+24)):
        start=offset(u32(names+index*4));result.append(data[start:data.index(0,start)].decode('ascii'))
    return {'path':str(path),'machine':hex(machine),'exports':sorted(result)}
def main():
    parser=argparse.ArgumentParser(__doc__);parser.add_argument('--kf1',type=Path,required=True);parser.add_argument('--generic',type=Path,required=True);args=parser.parse_args()
    report={'kf1':exports(args.kf1),'generic':exports(args.generic)}
    assert report['kf1']['machine']=='0x14c','KF1 requires x86'
    assert {'omath_kf1_start','omath_kf1_stop'}<=set(report['kf1']['exports'])
    assert {'omath_start_graphics_hooks','omath_stop_graphics_hooks','omath_set_graphics_callback'}<=set(report['generic']['exports'])
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
