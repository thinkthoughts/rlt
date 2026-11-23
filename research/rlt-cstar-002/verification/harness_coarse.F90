! RLT-CSTAR-002 harness. Compiles VERBATIM excerpts of ucla-roms 0.16.0
! src/roms_read_write.F90 init_coarse: L545-548, L564-575, L590-596 -> coarse_x.inc.
! (L550-562, the "coarse tile <= 2 points" abort, is evaluated separately: the
! ceiling width nxc0 is printed so the abort condition nxc0 <= 2 can be checked.)
! Output per (gnx, npx, inode): nxc0 (ceiling width), nxc (tile width), ic1-ic0+1, iSWc.
program harness_coarse
  implicit none
  integer(kind=4) :: gnx, gny, NP_XI, NP_ETA, inode, jnode, P, T, nxc0
  integer(kind=4) :: gnxc, gnyc, nxc, nyc, off_xc, iSWc
  integer(kind=4) :: iwestc, ieastc, jsouthc, jnorthc, ic0, ic1, jc0, jc1
  character(len=16) :: a
  call get_command_argument(1, a); read(a,*) P
  call get_command_argument(2, a); read(a,*) T
  gny = 2; NP_ETA = 1; jnode = 0
  do gnx = 2, P
    do NP_XI = 1, T
      do inode = 0, NP_XI-1
        nxc0 = (gnx/2 + NP_XI - 1)/NP_XI
        include 'coarse_x.inc'
        write(*,'(6I7)') gnx, NP_XI, inode, nxc0, nxc, ic1-ic0+1
      enddo
    enddo
  enddo
end program harness_coarse
