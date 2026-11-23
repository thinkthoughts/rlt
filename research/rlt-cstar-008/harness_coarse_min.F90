! RLT-CSTAR-008 replay: verbatim init_coarse lines (coarse_x.inc, ucla-roms 0.16.0
! src/roms_read_write.F90 L545-548, L564-575, L590-596). For each (gnx, NP_XI):
! nxc0 = ceiling width ROMS tests (abort if <= 2) and the smallest tile width nxc.
program harness_coarse_min
  implicit none
  integer(kind=4) :: gnx, gny, NP_XI, NP_ETA, inode, jnode, P, T, nxc0, nmin
  integer(kind=4) :: gnxc, gnyc, nxc, nyc, off_xc, iSWc
  integer(kind=4) :: iwestc, ieastc, jsouthc, jnorthc, ic0, ic1, jc0, jc1
  character(len=16) :: a
  call get_command_argument(1, a); read(a,*) P
  call get_command_argument(2, a); read(a,*) T
  gny = 2; NP_ETA = 1; jnode = 0
  do gnx = 2, P
    do NP_XI = 1, T
      nmin = huge(nmin)
      do inode = 0, NP_XI-1
        nxc0 = (gnx/2 + NP_XI - 1)/NP_XI
        include 'coarse_x.inc'
        nmin = min(nmin, nxc)
      enddo
      write(*,'(4I7)') gnx, NP_XI, nxc0, nmin
    enddo
  enddo
end program harness_coarse_min
