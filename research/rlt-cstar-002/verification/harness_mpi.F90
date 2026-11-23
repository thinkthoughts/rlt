! RLT-CSTAR-002 harness. Compiles VERBATIM excerpts of ucla-roms 0.16.0
! src/roms_mpi.F90 (L226-227 -> mpi_ceil.inc, L243-300 -> mpi_sizes.inc);
! only the surrounding declarations and loops are written here.
! Output per (gnx, npx, inode): interior width nx, xi_rho, xi_u, iSW_corn.
program harness_mpi
  implicit none
  integer(kind=4) :: gnx, gny, npx, npy, inode, jnode, NP_ETA, P, T
  integer(kind=4) :: nx, ny, offx, offy, iSW_corn, jSW_corn, Lm, Mm
  integer(kind=4) :: iwest, ieast, jsouth, jnorth, i0, i1, j0, j1
  integer(kind=4) :: xi_rho, xi_u, eta_rho, eta_v
  character(len=16) :: a
  call get_command_argument(1, a); read(a,*) P
  call get_command_argument(2, a); read(a,*) T
  gny = 1; npy = 1; jnode = 0; NP_ETA = 1
  do gnx = 1, P
    do npx = 1, T
      do inode = 0, npx-1
        include 'mpi_ceil.inc'
        include 'mpi_sizes.inc'
        write(*,'(5I7)') gnx, npx, inode, nx, xi_rho
        write(*,'(2I7)') xi_u, iSW_corn
      enddo
    enddo
  enddo
end program harness_mpi
