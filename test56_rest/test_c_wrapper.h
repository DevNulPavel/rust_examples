typedef enum
{
  MagickFalse = 0,
  MagickTrue = 1
} MagickBooleanType;

typedef enum
{
  UndefinedFilter,
  PointFilter,
  BoxFilter,
  TriangleFilter,
  HermiteFilter,
  HannFilter,
  HammingFilter,
  BlackmanFilter,
  GaussianFilter,
  QuadraticFilter,
  CubicFilter,
  CatromFilter,
  MitchellFilter,
  JincFilter,
  SincFilter,
  SincFastFilter,
  KaiserFilter,
  WelchFilter,
  ParzenFilter,
  BohmanFilter,
  BartlettFilter,
  LagrangeFilter,
  LanczosFilter,
  LanczosSharpFilter,
  Lanczos2Filter,
  Lanczos2SharpFilter,
  RobidouxFilter,
  RobidouxSharpFilter,
  CosineFilter,
  SplineFilter,
  LanczosRadiusFilter,
  CubicSplineFilter,
  SentinelFilter  /* a count of all the filters, not a real filter */
} FilterType;