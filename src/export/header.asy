import graph;
usepackage("amsmath");

size(122mm,90mm,IgnoreAspect);

path Circle = scale(1/sqrt(2)) * unitcircle;
path Square = shift(-0.5,-0.5) * unitsquare;
path Diamond = rotate(90) * Square;
path Cross = cross(4);
path Plus = rotate(45) * Cross;
path Up = polygon(3);
path Down = rotate(180) * polygon(3);
path Left = rotate(90) * polygon(3);
path Right = rotate(-90) * polygon(3);
path Asterisk = cross(6);
