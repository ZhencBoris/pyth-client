#include <stdio.h>
#include <math.h>
#include "../util/prng.h"

#define OVERLAP_MODEL_NEED_REF
#include "overlap_model.h"

int
main( int     argc,
      char ** argv ) {
  (void)argc; (void)argv;

  prng_t _prng[1];
  prng_t * prng = prng_join( prng_new( _prng, (uint32_t)0, (uint64_t)0 ) );

  long double max_ulp_fine   = 0.L;
  long double max_ulp_coarse = 0.L;

  long double const thresh_fine   = 2.6L;    /* max_rerr 2.4e-09 */
  long double const thresh_coarse = 32768.L; /* max_rerr 3.1e-05 */

  int ctr = 0;
  for( int iter=0; iter<100000000; iter++ ) {
    if( !ctr ) { printf( "Completed %u iterations\n", iter ); ctr = 10000000; }
    ctr--;

    /* Generate random tests that really stress price invariance assumptions */

    uint32_t t       = prng_uint32( prng );
    uint64_t mu_0    = prng_uint64( prng ) >> (t & (uint32_t)63); t >>= 6;
    uint64_t mu_1    = prng_uint64( prng ) >> (t & (uint32_t)63); t >>= 6;
    uint64_t sigma_0 = prng_uint64( prng ) >> (t & (uint32_t)63); t >>= 6;
    uint64_t sigma_1 = prng_uint64( prng ) >> (t & (uint32_t)63); t >>= 6;

    /* When sigmas are really dissimilar, limitations of the fixed point
       representation limit accuracy to ~2^15 ulp.  So we do separate
       states for when the two distributions have similar widths. */

    uint64_t sigma_min = sigma_0<sigma_1 ? sigma_0 : sigma_1;
    uint64_t sigma_max = sigma_0<sigma_1 ? sigma_1 : sigma_0;
    int fine = (sigma_min >= (sigma_max>>1)); /* sigmas of the two are comparable */

    long double y   = (long double)overlap_model( mu_0,sigma_0, mu_1,sigma_1 );
    long double z   = ((long double)(1<<30))*overlap_model_ref( (long double)mu_0,(long double)sigma_0,
                                                                (long double)mu_1,(long double)sigma_1 );
    long double ulp = fabsl( y - z );
    if( ulp>=(fine ? thresh_fine : thresh_coarse) ) {
      printf( "FAIL (iter %i: i(%lu,%lu) j(%lu,%lu) y %Lf z %Lf ulp %Lf)\n", iter, mu_0, sigma_0, mu_1, sigma_1, y, z, ulp );
      return 1;
    }
    if( fine && ulp>max_ulp_fine   ) max_ulp_fine   = ulp;
    if(         ulp>max_ulp_coarse ) max_ulp_coarse = ulp;
  }

  prng_delete( prng_leave( prng ) );

  printf( "pass (fine: max_rerr %.1Le max ulp %.1Lf, coarse: max_rerr %.1Le max ulp %.1Lf)\n",
          max_ulp_fine   / ((long double)(UINT64_C(1)<<30)), max_ulp_fine,
          max_ulp_coarse / ((long double)(UINT64_C(1)<<30)), max_ulp_coarse );

  return 0;
}

