"""Export canonical loading morph curves using Material + AndroidX, without Android.

Inputs live in target/material-progress-source; see assets/loading/README.md.
The tiny Java geometry adapters below replace Android Matrix/PointF/RectF only.
RoundedPolygon construction and Morph feature matching run in the upstream jar.
"""
from pathlib import Path
import re
import subprocess
import shutil
import hashlib

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "target/material-progress-source"
WORK = SOURCE / "export"
WORK.mkdir(exist_ok=True)
java = next(SOURCE.glob("*/Contents/Home/bin/java"))
javac = java.with_name("javac")
original = SOURCE / "lib/java/com/google/android/material/shape/MaterialShapes.java"
source = original.read_text()
source = re.sub(r"import (android\.|androidx.annotation\.).*;\n", "", source)
source = re.sub(r"@(?:NonNull|Nullable|RestrictTo\(Scope.LIBRARY_GROUP\))\s*", "", source)
source = re.sub(r"  public static ShapeDrawable createShapeDrawable\(.*?\n  }", "", source, flags=re.S)
source = source.replace("import androidx.graphics.shapes.Shapes_androidKt;", "")
source = source.replace("Shapes_androidKt.transformed", "Adapters.transformed")
(WORK / "MaterialShapes.java").write_text(source)
(WORK / "Adapters.java").write_text('''package com.google.android.material.shape;
import androidx.graphics.shapes.*;
import java.lang.reflect.Proxy;
class PointF {
  float x,y;
  PointF(float x,float y){this.x=x;this.y=y;}
  void offset(float x,float y){this.x+=x;this.y+=y;}
}
class RectF {
  float l,t,r,b;
  RectF(float l,float t,float r,float b){this.l=l;this.t=t;this.r=r;this.b=b;}
  float width(){return r-l;} float height(){return b-t;}
  float centerX(){return (l+r)*.5f;} float centerY(){return (t+b)*.5f;}
}
class Matrix {
  float a=1,b=0,c=0,d=1,x=0,y=0;
  void setScale(float a,float d){this.a=a;this.d=d;this.b=this.c=this.x=this.y=0;}
  void setRotate(float degrees){double v=Math.toRadians(degrees);a=d=(float)Math.cos(v);b=(float)Math.sin(v);c=-b;x=y=0;}
  void setSkew(float kx,float ky){a=d=1;b=ky;c=kx;x=y=0;}
  void preTranslate(float dx,float dy){x+=a*dx+c*dy;y+=b*dx+d*dy;}
  void postTranslate(float dx,float dy){x+=dx;y+=dy;}
}
class Adapters {
  static RoundedPolygon transformed(RoundedPolygon p, Matrix m) {
    PointTransformer fn=(PointTransformer)Proxy.newProxyInstance(PointTransformer.class.getClassLoader(),new Class[]{PointTransformer.class},(proxy,method,args)->{
      float x=(Float)args[0],y=(Float)args[1];
      return ((long)Float.floatToRawIntBits(m.a*x+m.c*y+m.x)<<32) | (Float.floatToRawIntBits(m.b*x+m.d*y+m.y)&0xffffffffL);
    });
    return p.transformed(fn);
  }
}
''')
(WORK / "Export.java").write_text('''package com.google.android.material.shape;
import androidx.graphics.shapes.*;
import java.io.*;
import java.nio.*;
import java.nio.file.*;
import java.util.*;
public class Export {
  static float[] points(Cubic c){return new float[]{c.getAnchor0X(),c.getAnchor0Y(),c.getControl0X(),c.getControl0Y(),c.getControl1X(),c.getControl1Y(),c.getAnchor1X(),c.getAnchor1Y()};}
  static String path(List<Cubic> cubics){
    StringBuilder s=new StringBuilder(); boolean first=true;
    for(Cubic c:cubics){float[] p=points(c);if(first){s.append("M "+p[0]+" "+p[1]+" ");first=false;}
      s.append("C "+p[2]+" "+p[3]+" "+p[4]+" "+p[5]+" "+p[6]+" "+p[7]+" ");}
    return s+"Z";
  }
  public static void main(String[] args)throws Exception{
    Path out=Path.of(args[0]);Files.createDirectories(out.resolve("reference"));
    RoundedPolygon[] shapes={MaterialShapes.SOFT_BURST,MaterialShapes.COOKIE_9,MaterialShapes.PENTAGON,MaterialShapes.PILL,MaterialShapes.SUNNY,MaterialShapes.COOKIE_4,MaterialShapes.OVAL};
    String[] names={"soft-burst","cookie-9","pentagon","pill","sunny","cookie-4","oval"};
    for(int i=0;i<7;i++)shapes[i]=MaterialShapes.normalize(shapes[i],true,new RectF(-1,-1,1,1));
    ByteBuffer data=ByteBuffer.allocate(100000).order(ByteOrder.LITTLE_ENDIAN);
    for(int i=0;i<7;i++){
      Morph morph=new Morph(shapes[i],shapes[(i+1)%7]);
      List<Cubic> a=morph.asCubics(0),b=morph.asCubics(1);data.putInt(a.size());
      for(int j=0;j<a.size();j++){for(float f:points(a.get(j)))data.putFloat(f);for(float f:points(b.get(j)))data.putFloat(f);}
      Files.writeString(out.resolve("reference/"+names[i]+".svg"),"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 48 48'><path transform='translate(24 24) rotate(-90) scale(17)' d='"+path(shapes[i].getCubics())+"'/></svg>\\n");
      // Independently generated midpoint checks protect Rust interpolation too.
      Files.writeString(out.resolve("reference/"+names[i]+"-midpoint.svg"),"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 48 48'><path transform='translate(24 24) rotate(-90) scale(17)' d='"+path(morph.asCubics(.5f))+"'/></svg>\\n");
      System.out.println(names[i]+": "+a.size()+" matched cubic pairs");
    }
    Files.write(out.resolve("morphs.bin"),Arrays.copyOf(data.array(),data.position()));
  }
}
''')
jars = [SOURCE / f for f in ["graphics-shapes.jar", "kotlin-stdlib.jar", "collection-jvm.jar"]]
classpath = ":".join(map(str, jars))
subprocess.run([str(javac), "-cp", classpath, "-d", str(WORK), *map(str, WORK.glob("*.java"))], check=True)
destination = ROOT / "assets/loading"
subprocess.run([str(java), "-cp", f"{WORK}:{classpath}", "com.google.android.material.shape.Export", str(destination)], check=True)
shutil.copyfile(SOURCE / "LICENSE", destination / "LICENSE-APACHE-2.0.txt")
files = [original, *jars, destination / "morphs.bin"]
(destination / "SHA256SUMS").write_text("".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(ROOT)}\n" for p in files))
