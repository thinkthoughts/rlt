! RLT-CSTAR-004 harness: function CSF of ucla-roms 0.16.0 src/set_scoord_mod.F90,
! lines 58-73 VERBATIM (csf.inc). Prints Cs at w-points (sigma = (k - N)/N) for
! N = 10 and a list of (theta_s, theta_b), in hexadecimal-free decimal with 17 digits.
module csf_mod
contains
  include 'csf.inc'
end module csf_mod
program harness_csf
  use csf_mod
  implicit none
  real(kind=8) :: ts(6), tb(6), sc
  integer :: i, k, n
  ts = (/ 0.d0, 0.d0, 5.d0, 5.d0, 1.d-4, 10.d0 /)
  tb = (/ 0.d0, 2.d0, 0.d0, 2.d0, 1.d-4, 10.d0 /)
  n = 10
  do i = 1, 6
    do k = 0, n
      sc = dble(k - n)/dble(n)
      write(*,'(3ES26.17E3)') ts(i), tb(i), CSF(sc, ts(i), tb(i))
    enddo
  enddo
end program harness_csf
