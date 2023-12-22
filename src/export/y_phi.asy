real phimin = -pi-0.1;
real phimax = pi+0.1;

string phi_label(real x){
  static real step = pi/2;
  static string[] labels = new string[]{
    "$-\pi$", "$-\tfrac{\pi}{2}$", "$0$", "$\tfrac{\pi}{2}$", "$\pi$",
  };
  int num = round((x - phimin)/step);
  if(num > labels.length) return "";
  return labels[num];
}

guide jet_guide(real y, real phi, real R) {
  pair centre = (theta_bar(y), phi);
  return graph(
               new pair(real alpha) {
                 return (theta_bar(y + R*cos(alpha)), phi + R*sin(alpha));
               },
               0, 2*pi
               )..cycle;
}
