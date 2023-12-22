import graph;
usepackage("amsmath");

real goldenratio=(1+sqrt(5))/2;
size(9cm*goldenratio,9cm,IgnoreAspect);

string y_label(real theta_bar){
  if(theta_bar == 0) return "$-\infty$";
  if(theta_bar == pi) return "$\infty$";
  real y = -2*log(tan((pi - theta_bar)/2));
  int yint = round(y);
  return (string) yint;
}

real theta(real y) {
  return 2*atan(exp(-y));
}

real theta_bar(real y) {
  return pi - theta(y/2);
}

real[] yTicks = new real[]{0,theta_bar(-5),theta_bar(-4),theta_bar(-3),theta_bar(-2),theta_bar(-1),theta_bar(0),theta_bar(1),theta_bar(2),theta_bar(3),theta_bar(4),theta_bar(5),pi};
real[] yticks = new real[]{theta_bar(-7/2),theta_bar(-5/2),theta_bar(-3/2),theta_bar(-1/2),theta_bar(1/2),theta_bar(3/2),theta_bar(5/2),theta_bar(7/2)};

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

real xmin = -0.02;
real xmax = pi+0.02;
